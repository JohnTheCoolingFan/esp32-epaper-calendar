//! Use <https://www.isdayoff.ru> to get info on what calendar days are days off
//! Only works for Belarus, Kazakhstan, Russia and Ukraine. Alternatvie providers might be a good
//! feature.

use alloc::format;
use core::str::from_utf8;

use arrayvec::ArrayString;
use chrono::{Month, Months};
use embassy_net::{dns::DnsSocket, tcp::client::TcpClient};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use heapless::LinearMap;
use log::error;
use reqwless::{client::HttpClient, request::Method, response::StatusCode};

use crate::calendar_utils::{CalendarMonth, DaysOffMask, MonthDate};

/// Country to fetch the isdayoff data for
const TARGET_COUNTRY: TargetCountry = TargetCountry::Russia;

static ISDAYOFF_CACHE: Mutex<
    CriticalSectionRawMutex,
    heapless::LinearMap<MonthDate, DaysOffMask, 3>,
> = Mutex::new(LinearMap::new());

async fn insert_cache(month: MonthDate, mask: DaysOffMask) {
    let mut cache = ISDAYOFF_CACHE.lock().await;
    let _ = cache.insert(month, mask).inspect_err(|_e| {
        error!(
            "Failed to populate cache year {} month {}, attempt to insert over capacity",
            month.year(),
            month.month().number_from_month()
        )
    });
}

async fn remove_cache(month: MonthDate) {
    let mut cache = ISDAYOFF_CACHE.lock().await;
    let _ = cache.remove(&month);
}

pub async fn populate_cache(
    client: &mut HttpClientConcrete,
    current_month: MonthDate,
) -> Result<(), reqwless::Error> {
    let months = get_months_triplet(current_month);

    for month in months {
        let days_off_mask = get_days_off_mask(client, month).await?;
        if let Some(mask) = days_off_mask {
            let mask = DaysOffMask::new(mask);
            insert_cache(month, mask).await;
        }
    }
    Ok(())
}

pub async fn clear_cache() {
    let mut cache = ISDAYOFF_CACHE.lock().await;
    cache.clear();
}

pub async fn rotate_cache(client: &mut HttpClientConcrete, current_month: MonthDate) {
    remove_cache(current_month - Months::new(2)).await;
    let next_month = current_month + Months::new(1);
    let next_month_mask = get_days_off_mask(client, next_month).await.unwrap();
    if let Some(mask) = next_month_mask {
        let mask = DaysOffMask::new(mask);
        insert_cache(next_month, mask).await;
    }
}

fn get_months_triplet(current_month: MonthDate) -> [MonthDate; 3] {
    [
        current_month - Months::new(1),
        current_month,
        current_month + Months::new(1),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetCountry {
    Belarus,
    Kazakhstan,
    Russia,
    Ukraine,
}

impl TargetCountry {
    pub const fn to_countrycode(self) -> &'static str {
        match self {
            Self::Belarus => "by",
            Self::Kazakhstan => "kz",
            Self::Russia => "ru",
            Self::Ukraine => "ua",
        }
    }
}

pub type HttpClientConcrete =
    HttpClient<'static, TcpClient<'static, 1, 4096, 4096>, DnsSocket<'static>>;

pub async fn update_days_off_mask(
    client: &mut HttpClientConcrete,
    calendar: &mut CalendarMonth,
) -> Result<(), reqwless::Error> {
    let mask = get_days_off_mask(client, calendar.month_date()).await?;
    calendar.set_days_off(DaysOffMask::new(mask.unwrap()));
    Ok(())
}

pub async fn get_days_off_mask(
    client: &mut HttpClientConcrete,
    date: MonthDate,
) -> Result<Option<u32>, reqwless::Error> {
    let year = date.year();
    let month = date.month().number_from_month();
    info!("Fetching isdayoff data for year {year} month {month}");
    let cc = TARGET_COUNTRY.to_countrycode();
    let url = format!("http://isdayoff.ru/api/getdata?year={year}&month={month}&cc={cc}");
    let mut rx_buf = [0; 4096];
    let mut request = client.request(Method::GET, &url).await?;
    let response = request.send(&mut rx_buf).await?;

    match response.status {
        StatusCode(200) => {
            let body = response.body().read_to_end().await.unwrap();
            body.reverse();
            let body = from_utf8(&*body).unwrap();
            let res = parse_isdayoff_response(body);
            Ok(Some(res))
        }
        StatusCode(400) => {
            let body = response.body().read_to_end().await.unwrap();
            // Why is this service using 400 as status code for "service error"? It should be 5XX.
            // It even uses 400 for "not found" like favicon.ico!
            if body == b"100" {
                log::error!("isdayoff request failed, invalid date");
            } else if body == b"199" {
                log::error!("isdayoff request failed, backend error");
            } else {
                let body = from_utf8(&*body).unwrap();
                log::warn!("Unexpected error response: {body}")
            }
            Ok(None)
        }
        StatusCode(404) => {
            log::error!("isdayoff found no data");
            Ok(None)
        }
        _ => {
            log::warn!("Unexpected status code: {}", response.status.0);
            Ok(None)
        }
    }
}

fn parse_isdayoff_response(body: &str) -> u32 {
    u32::from_str_radix(body, 2).unwrap()
}
