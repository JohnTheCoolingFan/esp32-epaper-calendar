#![no_std]
#![no_main]

use core::cell::RefCell;

use calendar_utils::CalendarMonth;
#[cfg(feature = "calendar-style-triplet")]
use chrono::Months;
use chrono::{Days, NaiveTime};
#[allow(unused_imports)]
use defmt::{debug, error, info, trace, warn};
use display_interface_spi::SPIInterface;
use draw::calendar::draw_calendars;
use ds323x::{Ds323x, ic::DS3231, interface::I2cInterface};
use embassy_embedded_hal::shared_bus::{asynch::spi::SpiDevice, blocking::i2c::I2cDevice};
use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::{self, raw::CriticalSectionRawMutex},
    mutex::Mutex,
};
use embassy_time::Timer;
use esp_backtrace as _;
use esp_hal::{
    Async, Blocking,
    clock::CpuClock,
    dma::{DmaChannel, DmaPriority, DmaRxBuf, DmaTxBuf},
    dma_buffers,
    gpio::{Input, Level, NoPin, Output, Pull},
    i2c::{self, master::I2c},
    rng::Rng,
    spi::master::{Config, Spi, SpiDmaBus},
    time::RateExtU32,
};
use esp_hal_embassy::main;
#[cfg(feature = "isdayoff")]
use http_apis::isdayoff::update_days_off_mask;
#[cfg(feature = "ntp")]
use time::ntp::synchronize_ntp_time_to_rtc;
use time::{RTC_CLOCK, get_local_rtc_time};

extern crate alloc;

use weact_studio_epd::{
    TriColor, WeActStudio290TriColorDriver,
    graphics::{Display290TriColor, DisplayRotation},
};

mod calendar_utils;
mod draw;
#[cfg(feature = "http_apis")]
mod http_apis;
mod time;
#[cfg(feature = "networking")]
mod wifi;

pub type SpiBusMutex = Mutex<CriticalSectionRawMutex, SpiDmaBus<'static, Async>>;
pub type I2cBusMutex =
    blocking_mutex::Mutex<CriticalSectionRawMutex, RefCell<I2c<'static, Blocking>>>;
pub type Ds323xTypeConcrete = Ds323x<
    I2cInterface<I2cDevice<'static, CriticalSectionRawMutex, I2c<'static, Blocking>>>,
    DS3231,
>;
pub type RtcDs323x = blocking_mutex::Mutex<CriticalSectionRawMutex, RefCell<Ds323xTypeConcrete>>;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.init_with(|| $val);
        x
    }};
}

#[cfg(not(any(feature = "calendar-style-bignum", feature = "calendar-style-triplet")))]
compile_error!("Configure one of the styles: `calendar-style-bignum` or `calendar-style-triplet`!");
#[cfg(all(feature = "calendar-style-bignum", feature = "calendar-style-triplet"))]
compile_error!("Configure only one style!");

#[cfg(all(
    feature = "networking",
    not(any(feature = "isdayoff", feature = "ntp", feature = "weather"))
))]
compile_error!(
    "Networking is enabled, but nothing that requires networking is enabled.\nDo not enable the networking feature manually, it's just a common dependency for other features that require networking."
);
#[cfg(all(
    feature = "http_apis",
    not(any(feature = "isdayoff", feature = "weather"))
))]
compile_error!(
    "http_apis feature is enabled, but nothing that requires http api access is enabled.\nDo not enable the http_apis feature manually, it's just a common dependency for other features that call to HTTP APIs."
);

#[main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_alloc::heap_allocator!(72 * 1024);

    let delay = embassy_time::Delay;

    let timg1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timg1.timer0);

    info!("Embassy initialized!");

    info!("RNG init");

    let rng = Rng::new(peripherals.RNG);

    #[cfg(feature = "networking")]
    let net_stack = {
        let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);

        let wifi_interface = wifi::init_wifi(
            &spawner,
            timg0,
            rng,
            peripherals.RADIO_CLK,
            peripherals.WIFI,
        );

        wifi::init_networking(&spawner, rng, wifi_interface)
    };

    #[cfg(feature = "http_apis")]
    let http_client = http_apis::init_tcp_http(net_stack);

    info!("Initializing I2C");

    let i2c_bus = mk_static!(I2cBusMutex, {
        let i2c_bus = I2c::new(
            peripherals.I2C0,
            i2c::master::Config::default().with_frequency(400_u32.kHz()),
        )
        .unwrap()
        .with_sda(peripherals.GPIO11)
        .with_scl(peripherals.GPIO12);
        blocking_mutex::Mutex::<CriticalSectionRawMutex, _>::new(RefCell::new(i2c_bus))
    });
    let i2c_dev_ds323x = I2cDevice::new(&*i2c_bus);

    info!("Initializing DS3231 external RTC");

    // At this point the RTC_CLOCK is not yet initialized, guaranteed to be initialized HERE. Any
    // usage must be AFTER this.
    #[allow(unused_variables)]
    let rtc = RTC_CLOCK.get_or_init(|| {
        let mut rtc = Ds323x::new_ds3231(i2c_dev_ds323x);
        rtc.enable().unwrap();
        rtc.disable_32khz_output().unwrap();
        blocking_mutex::Mutex::new(RefCell::new(rtc))
    });

    info!("Initializing spi pins");

    let cs = Output::new(peripherals.GPIO5, Level::High);
    let busy_in = Input::new(peripherals.GPIO4, Pull::Up);
    let rst = Output::new(peripherals.GPIO10, Level::High);
    let dc = Output::new(peripherals.GPIO17, Level::Low);

    info!("Initializing spi bus");

    let dma_channel = peripherals.DMA_CH2;
    dma_channel.set_priority(DmaPriority::Priority0);

    #[allow(clippy::manual_div_ceil)]
    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let spi_bus = mk_static!(SpiBusMutex, {
        let spi_dma_bus: SpiDmaBus<'static, Async> = Spi::<'static, _>::new(
            peripherals.SPI2,
            // the display controller supports SPI speed up to 20MHz
            Config::default().with_frequency(20_u32.MHz()),
        )
        .unwrap()
        .with_cs(NoPin)
        .with_miso(NoPin)
        .with_sck(peripherals.GPIO18)
        .with_mosi(peripherals.GPIO21)
        .with_dma(dma_channel)
        .with_buffers(dma_rx_buf, dma_tx_buf)
        .into_async();
        Mutex::<CriticalSectionRawMutex, _>::new(spi_dma_bus)
    });

    info!("Initializing spi device");

    let spi_device = SpiDevice::new(spi_bus, cs);
    let spi_interface = SPIInterface::new(spi_device, dc);

    info!("Initializing epd");

    let mut driver = WeActStudio290TriColorDriver::new(spi_interface, busy_in, rst, delay);
    driver.init().await.unwrap();

    info!("Display buffer init");

    let mut display = Display290TriColor::new();
    display.set_rotation(DisplayRotation::Rotate90);

    info!("Display buffer init done");

    // TODO: Spawn some tasks
    let _ = spawner;

    info!("Loop starting");

    loop {
        // todo: time sync
        // todo: fetch and use isdayoff

        #[cfg(feature = "ntp")]
        {
            info!("NTP time sync");
            synchronize_ntp_time_to_rtc(net_stack).await;
        }

        info!("Getting time");
        let local_time = get_local_rtc_time().unwrap();
        #[allow(unused_mut)]
        let mut calendar = CalendarMonth::from_date(local_time.date_naive());
        #[allow(unused_mut)]
        #[cfg(feature = "calendar-style-triplet")]
        let mut calendar_before =
            CalendarMonth::from_date(local_time.date_naive() - Months::new(1));
        #[allow(unused_mut)]
        #[cfg(feature = "calendar-style-triplet")]
        let mut calendar_after = CalendarMonth::from_date(local_time.date_naive() + Months::new(1));

        #[cfg(feature = "isdayoff")]
        {
            net_stack.wait_config_up().await;
            info!("Getting isdayoff data");
            update_days_off_mask(http_client, &mut calendar)
                .await
                .unwrap();
            #[cfg(feature = "calendar-style-triplet")]
            update_days_off_mask(http_client, &mut calendar_before)
                .await
                .unwrap();
            #[cfg(feature = "calendar-style-triplet")]
            update_days_off_mask(http_client, &mut calendar_after)
                .await
                .unwrap();
        }

        #[cfg(all(feature = "calendar-style-bignum", feature = "weather"))]
        let forecast = {
            net_stack.wait_config_up().await;
            info!("Getting weather forecast");
            http_apis::weather::get_weather_forecast(http_client)
                .await
                .inspect_err(|err| {
                    use defmt::Display2Format;

                    error!(
                        "Error getting weather forecast data: {}",
                        Display2Format(err)
                    )
                })
                .ok()
        };

        info!("Drawing calendar");
        display.clear(TriColor::White);
        draw_calendars(
            &local_time,
            calendar,
            #[cfg(feature = "calendar-style-triplet")]
            calendar_before,
            #[cfg(feature = "calendar-style-triplet")]
            calendar_after,
            &mut display,
        )
        .unwrap();
        #[cfg(all(feature = "calendar-style-bignum", feature = "weather"))]
        if let Some(forecast) = forecast {
            draw::weather::draw_forecast(forecast, &mut display).unwrap();
        }
        driver.wake_up().await.unwrap();
        driver.full_update(&display).await.unwrap();
        driver.sleep().await.unwrap();

        info!("Getting time and sleeping");
        let local_time = get_local_rtc_time().unwrap();

        // Wait until 00:00:05 of the next day
        let wait_time = ((local_time + Days::new(1))
            .with_time(NaiveTime::from_hms_opt(0, 0, 5).unwrap())
            .unwrap()
            - local_time)
            .num_seconds();

        Timer::after_secs(wait_time.try_into().unwrap()).await;
        info!("Wake up from waiting");
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.22.0/examples/src/bin
}
