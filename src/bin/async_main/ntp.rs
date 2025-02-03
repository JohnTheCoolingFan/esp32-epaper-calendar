use chrono::NaiveDateTime;
use ds323x::DateTimeAccess;
use sntpc::NtpTimestampGenerator;

use crate::RtcDs323x;

#[derive(Clone, Copy)]
pub struct TimestampGenerator {
    timestamp: NaiveDateTime,
    pub rtc: &'static RtcDs323x,
}

impl TimestampGenerator {
    fn new(rtc: &'static RtcDs323x) -> Self {
        Self {
            timestamp: Default::default(),
            rtc,
        }
    }
}

impl NtpTimestampGenerator for TimestampGenerator {
    fn init(&mut self) {
        let new_timestamp = self.rtc.lock(|rtc| rtc.borrow_mut().datetime().unwrap());
        self.timestamp = new_timestamp
    }

    fn timestamp_sec(&self) -> u64 {
        self.timestamp.and_utc().timestamp().try_into().unwrap()
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        self.timestamp.and_utc().timestamp_subsec_micros()
    }
}
