use chrono::NaiveDateTime;
use ds323x::DateTimeAccess;
use log::error;
use sntpc::NtpTimestampGenerator;

use crate::RTC_CLOCK;

#[derive(Clone, Copy)]
pub struct TimestampGenerator {
    timestamp: NaiveDateTime,
}

impl NtpTimestampGenerator for TimestampGenerator {
    fn init(&mut self) {
        if let Some(rtc) = RTC_CLOCK.try_get() {
            let new_timestamp = rtc.lock(|rtc| rtc.borrow_mut().datetime().unwrap());
            self.timestamp = new_timestamp
        } else {
            error!("RTC_CLOCK is not set!");
        }
    }

    fn timestamp_sec(&self) -> u64 {
        self.timestamp.and_utc().timestamp().try_into().unwrap()
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        self.timestamp.and_utc().timestamp_subsec_micros()
    }
}
