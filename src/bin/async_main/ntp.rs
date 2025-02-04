use chrono::NaiveDateTime;
use ds323x::DateTimeAccess;
use embassy_sync::once_lock::OnceLock;
use log::error;
use sntpc::NtpTimestampGenerator;

pub static RTC_CLOCK: OnceLock<RtcDs323x> = OnceLock::new();

#[derive(Debug)]
pub enum RtcClockError {
    // The field is used when debug-printing on error
    I2cClockError(#[allow(dead_code)] <Ds323xTypeConcrete as DateTimeAccess>::Error),
    ClockCellNotSet,
}

/// Get time from the RTC clock on the I2C bus
pub fn get_rtc_time() -> Result<NaiveDateTime, RtcClockError> {
    RTC_CLOCK
        .try_get()
        .ok_or(RtcClockError::ClockCellNotSet)
        .inspect_err(|_e| {
            error!("RTC_CLOCK is not set!");
        })?
        .lock(|rtc_lock| rtc_lock.borrow_mut().datetime())
        .map_err(RtcClockError::I2cClockError)
        .inspect_err(|e| {
            error!("RTC clock error: {e:?}");
        })
}

use crate::{Ds323xTypeConcrete, RtcDs323x};

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
