//! Use <https://www.isdayoff.ru> to get info on what calendar days are days off
//! Only works for Belarus, Kazakhstan, Russia and Ukraine. Alternatvie providers might be a good
//! feature.

use alloc::format;
use core::str::from_utf8;

use arrayvec::ArrayString;
use chrono::Month;
use embassy_net::{dns::DnsSocket, tcp::client::TcpClient};
use reqwless::{client::HttpClient, request::Method, response::StatusCode};

/// Country to fetch the isdayoff data for
const TARGET_COUNTRY: TargetCountry = TargetCountry::Russia;

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

pub async fn get_days_off_mask(
    client: &mut HttpClient<'static, TcpClient<'static, 1, 4096, 4096>, DnsSocket<'static>>,
    year: u16,
    month: u8,
) -> Result<Option<u32>, reqwless::Error> {
    let cc = TARGET_COUNTRY.to_countrycode();
    let url = format!("https://isdayoff.ru/api/getdata?year={year}&month={month}&cc={cc}");
    let mut rx_buf = [0; 4096];
    let mut request = client.request(Method::GET, &url).await?;
    let response = request.send(&mut rx_buf).await?;

    match response.status {
        StatusCode(200) => {
            let body = response.body().read_to_end().await.unwrap();
            body.reverse();
            let body = from_utf8(&*body).unwrap();
            let body = ArrayString::<32>::from(body).unwrap();
            let res = parse_isdayoff_response(body);
            Ok(Some(res))
        }
        StatusCode(400) => {
            let body = response.body().read_to_end().await.unwrap();
            let body = from_utf8(&*body).unwrap();
            // Why is this service using 400 as status code for "service error"? It should be 5XX.
            // It even uses 400 for "not found" like favicon.ico!
            if body == "100" {
                log::error!("isdayoff request failed, invalid date");
            } else if body == "199" {
                log::error!("isdayoff request failed, backend error");
            } else {
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

fn parse_isdayoff_response(body: ArrayString<32>) -> u32 {
    u32::from_str_radix(&body, 2).unwrap()
}
