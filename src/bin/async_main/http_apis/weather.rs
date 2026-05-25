use alloc::format;
use core::{cmp::Ordering, error::Error, fmt::Display};

use reqwless::response::StatusCode;
use serde::Deserialize;

use super::HttpClientConcrete;

// Inspired by AlexGyver's RipCalendar (https://github.com/AlexGyver/RipCalendar)

// ---- User-specified constants ---- //

const LATITUDE: &str = env!("WEATHER_LATITUDE");
const LONGITUDE: &str = env!("WEATHER_LONGITUDE");

// ---- Return data type ---- //

#[derive(Debug, Clone)]
pub struct ForecastSummary {
    /// 6:00 .. 12:00
    pub morning: ForecastSummaryPeriod,
    /// 12:00 .. 18:00
    pub day: ForecastSummaryPeriod,
    /// 18:00 .. 23:00
    pub evening: ForecastSummaryPeriod,
}

#[derive(Debug, Clone)]
pub struct ForecastSummaryPeriod {
    // pub timestamp: u64, // implied from index
    pub apparent_temperature_range: (f32, f32), /* f16 would suffice, but seems the xtensa cpu
                                                 * can only do f32 */
    pub wind_speed_avg: f32,
    pub weather_code_max: u8,
}

// ---- Main method ---- //

pub async fn get_weather_forecast(
    client: &mut HttpClientConcrete,
) -> Result<ForecastSummary, ForecastError> {
    let url = format!(
        "http://api.open-meteo.com/v1/forecast?longitude={LONGITUDE}&latitude={LATITUDE}&hourly=apparent_temperature,weather_code,wind_speed_10m&wind_speed_unit=ms&timeformat=unixtime&timezone=auto&forecast_days=1"
    );

    let mut rx_buf = [0_u8; 4096];
    let mut request = client.request(reqwless::request::Method::GET, &url).await?;
    let response = request.send(&mut rx_buf).await?;

    match response.status {
        StatusCode(200) => {
            let body = response.body().read_to_end().await?;
            let (response_parsed, _) = serde_json_core::from_slice::<OpenMeteoResponse>(&*body)?;
            Ok(response_parsed.summarize())
        }
        _ => Err(ForecastError::StatusCode(response.status)),
    }
}

// ---- API data parsing ---- //

// only includes fields relevant to the working of this module
#[derive(Debug, Clone, Deserialize)]
struct OpenMeteoResponse {
    hourly: HourlyData,
}

#[derive(Debug, Clone, Deserialize)]
struct HourlyData {
    // timestamp is skipped: will be implied to be a start of the day and each hour from that
    apparent_temperature: [f32; 24],
    wind_speed_10m: [f32; 24],
    weather_code: [u8; 24],
}

// Restricted to use within this module, the data from the API will not have NaNs or infinity
fn f32_cmp(a: &&f32, b: &&f32) -> Ordering {
    // SAFETY: data from the api is guaranteed to not have NaN or infinity, that would
    // make zero sense
    unsafe { a.partial_cmp(b).unwrap_unchecked() }
}

impl OpenMeteoResponse {
    fn summarize(self) -> ForecastSummary {
        let [morning, day, evening] = [(6..12), (12..18), (12..18)].map(|time_range| {
            let (temps, winds, wcodes) = (
                &self.hourly.apparent_temperature[time_range.clone()],
                &self.hourly.wind_speed_10m[time_range.clone()],
                &self.hourly.weather_code[time_range],
            );
            let temp_max = *temps
                .iter()
                .max_by(f32_cmp)
                .expect("Temperature array must not be empty");
            let temp_min = *temps
                .iter()
                .min_by(f32_cmp)
                .expect("Temperature array must not be empty");
            let code_max = *wcodes
                .iter()
                .max()
                .expect("Weather codes array must not be empty");
            let wind_avg = winds.iter().copied().sum::<f32>() / winds.len() as f32;

            ForecastSummaryPeriod {
                apparent_temperature_range: (temp_min, temp_max),
                wind_speed_avg: wind_avg,
                weather_code_max: code_max,
            }
        });

        ForecastSummary {
            morning,
            day,
            evening,
        }
    }
}

// ---- Error type ---- //

#[derive(Debug)]
pub enum ForecastError {
    Parse(serde_json_core::de::Error),
    Request(reqwless::Error),
    StatusCode(StatusCode),
}

impl Display for ForecastError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "HTTP Response parsing error: {err}"),
            Self::Request(err) => write!(f, "HTTP Request error: {err:?}"),
            Self::StatusCode(status) => write!(f, "Unsuccessful status code: {}", status.0),
        }
    }
}

impl Error for ForecastError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Parse(err) => Some(err),
            // reqwless error doesn't implement core:error::Error until 0.14
            Self::Request(_) | Self::StatusCode(_) => None,
        }
    }
}

impl From<reqwless::Error> for ForecastError {
    fn from(value: reqwless::Error) -> Self {
        Self::Request(value)
    }
}

impl From<serde_json_core::de::Error> for ForecastError {
    fn from(value: serde_json_core::de::Error) -> Self {
        Self::Parse(value)
    }
}
