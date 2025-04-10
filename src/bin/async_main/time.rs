use core::{net::SocketAddr, ops::DerefMut};

use chrono::{DateTime, NaiveDateTime, TimeDelta};
use chrono_tz::Tz;
use ds323x::DateTimeAccess;
use embassy_net::{
    udp::{PacketMetadata, UdpSocket},
    Stack,
};
use embassy_sync::once_lock::OnceLock;
use log::error;
use smoltcp::wire::DnsQueryType;
use sntpc::{NtpContext, NtpTimestampGenerator};

use crate::{Ds323xTypeConcrete, RtcDs323x};

pub static RTC_CLOCK: OnceLock<RtcDs323x> = OnceLock::new();

/// Change this value to change the local timezone
///
/// Used to synchronize the day roll-over time
pub const LOCAL_TZ: chrono_tz::Tz = chrono_tz::Europe::Moscow;

#[derive(Debug)]
pub enum RtcClockError {
    // The field is used when debug-printing on error
    I2cClockError(#[allow(dead_code)] <Ds323xTypeConcrete as DateTimeAccess>::Error),
    ClockCellNotSet,
}

/// Convenience wrapper to access the I2C bus attached external RTC that is gated behind all those
/// locks and mutexes, with error messaging.
pub fn access_rtc_clock<T, F>(f: F) -> Result<T, RtcClockError>
where
    F: FnOnce(&mut Ds323xTypeConcrete) -> Result<T, <Ds323xTypeConcrete as DateTimeAccess>::Error>,
{
    RTC_CLOCK
        .try_get()
        .ok_or(RtcClockError::ClockCellNotSet)
        .inspect_err(|_e| {
            error!("RTC_CLOCK is not set!");
        })?
        .lock(|rtc_lock| {
            let mut rtc_borrow = rtc_lock.borrow_mut();
            f(rtc_borrow.deref_mut())
        })
        .map_err(RtcClockError::I2cClockError)
        .inspect_err(|e| {
            error!("RTC clock error: {e:?}");
        })
}

/// Get time from the RTC clock
pub fn get_rtc_time() -> Result<NaiveDateTime, RtcClockError> {
    access_rtc_clock(|rtc| rtc.datetime())
}

/// Get local tiem from RTC
pub fn get_local_rtc_time() -> Result<DateTime<Tz>, RtcClockError> {
    get_rtc_time().map(|dt| dt.and_utc().with_timezone(&LOCAL_TZ))
}

/// Set the RTC module time
pub fn set_rtc_clock(new_datetime: &NaiveDateTime) -> Result<(), RtcClockError> {
    access_rtc_clock(|rtc| rtc.set_datetime(new_datetime))
}

#[derive(Clone, Copy, Default)]
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

const NTP_SERVER_POOL: &[&str] = &["pool.ntp.org"];
const NTP_PORT: u16 = 123;

pub async fn try_resolve_from_pool(
    stack: Stack<'_>,
    pool: &'static [&'static str],
) -> Option<(
    &'static str,
    heapless::Vec<smoltcp::wire::IpAddress, { smoltcp::config::DNS_MAX_RESULT_COUNT }>,
)> {
    for address in pool {
        match stack.dns_query(address, DnsQueryType::A).await {
            Ok(res) => {
                if res.is_empty() {
                    log::warn!("No IP addresses returned for NTP server `{address}`");
                } else {
                    return Some((address, res));
                }
            }
            Err(e) => {
                error!("Failed to query IP address for NTP server `{address}`: {e:?}");
            }
        }
    }
    None
}

/// Get time from an NTP server
pub async fn get_ntp_time(stack: Stack<'_>) -> Option<NaiveDateTime> {
    stack.wait_config_up().await;
    let (ntp_server_name, ntp_addresses) = try_resolve_from_pool(stack, NTP_SERVER_POOL).await?;

    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 4096];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    socket.bind(NTP_PORT).unwrap();

    let ntp_context = NtpContext::new(TimestampGenerator::default());

    for address in ntp_addresses {
        let ntp_result =
            sntpc::get_time(SocketAddr::from((address, NTP_PORT)), &socket, ntp_context).await;

        match ntp_result {
            Ok(time) => {
                let new_datetime = NaiveDateTime::UNIX_EPOCH
                    + TimeDelta::new(
                        time.sec().into(),
                        sntpc::fraction_to_nanoseconds(time.sec_fraction()),
                    )
                    .unwrap();
                return Some(new_datetime);
            }
            Err(e) => {
                error!("Failed to synchronize time from server `{ntp_server_name}` at IP `{address}`: {e:?}");
            }
        }
    }
    None
}

/// Set RTC time to what we get from an NTP server
pub async fn synchronize_ntp_time_to_rtc(net_stack: Stack<'_>) {
    let network_time = get_ntp_time(net_stack).await;
    if let Some(new_time) = network_time {
        set_rtc_clock(&new_time).unwrap();
    } else {
        error!("Failed to synchronize time over the network");
    }
}
