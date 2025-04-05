//! Use <https://www.isdayoff.ru> to get info on what calendar days are days off
//! Only works for Belarus, Kazakhstan, Russia and Ukraine. Alternatvie providers might be a good
//! feature.

use alloc::format;
use core::str::from_utf8;

use arrayvec::ArrayString;
use chrono::Month;
use embassy_net::{dns::DnsSocket, tcp::client::TcpClient};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use heapless::LinearMap;
use reqwless::{client::HttpClient, request::Method, response::StatusCode};

use crate::calendar_utils::{CalendarMonth, DaysOffMask, MonthDate};

/// Country to fetch the isdayoff data for
const TARGET_COUNTRY: TargetCountry = TargetCountry::Russia;

static ISDAYOFF_CACHE: Mutex<
    CriticalSectionRawMutex,
    heapless::LinearMap<MonthDate, DaysOffMask, 3>,
> = Mutex::new(LinearMap::new());

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
    let year = calendar.year();
    let month = calendar.month() as u8 + 1;
    let mask = get_days_off_mask(client, year, month).await?;
    calendar.set_days_off(DaysOffMask::new(mask.unwrap()));
    Ok(())
}

pub async fn get_days_off_mask(
    client: &mut HttpClientConcrete,
    year: u16,
    month: u8,
) -> Result<Option<u32>, reqwless::Error> {
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
