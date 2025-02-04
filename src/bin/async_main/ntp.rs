use chrono::NaiveDateTime;
use ds323x::DateTimeAccess;
use log::error;
use sntpc::NtpTimestampGenerator;

use crate::{get_rtc_time, RTC_CLOCK};

#[derive(Clone, Copy)]
pub struct TimestampGenerator {
    timestamp: NaiveDateTime,
}

impl NtpTimestampGenerator for TimestampGenerator {
    fn init(&mut self) {
        self.timestamp = get_rtc_time().unwrap();
    }

    fn timestamp_sec(&self) -> u64 {
        self.timestamp.and_utc().timestamp().try_into().unwrap()
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        self.timestamp.and_utc().timestamp_subsec_micros()
    }
}
