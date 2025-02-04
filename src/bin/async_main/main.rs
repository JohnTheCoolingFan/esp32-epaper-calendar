#![no_std]
#![no_main]

use core::cell::RefCell;

use chrono::NaiveDateTime;
use display_interface_spi::SPIInterface;
use ds323x::{ic::DS3231, interface::I2cInterface, Ds323x};
use embassy_embedded_hal::shared_bus::{asynch::spi::SpiDevice, blocking::i2c::I2cDevice};
use embassy_executor::Spawner;
use embassy_net::StackResources;
use embassy_sync::{
    blocking_mutex::{self, raw::CriticalSectionRawMutex},
    mutex::Mutex,
    once_lock::OnceLock,
};
use embassy_time::Timer;
use embedded_graphics::{
    mono_font::MonoTextStyle,
    prelude::*,
    text::{Text, TextStyle},
    Drawable,
};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    dma::{DmaChannel, DmaPriority, DmaRxBuf, DmaTxBuf},
    dma_buffers,
    gpio::{Input, Level, NoPin, Output, Pull},
    i2c::{self, master::I2c},
    rng::Rng,
    spi::master::{Config, Spi, SpiDmaBus},
    time::RateExtU32,
    Async, Blocking,
};
use esp_hal_embassy::main;
use esp_wifi::{wifi::WifiStaDevice, EspWifiController};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use profont::PROFONT_24_POINT;

extern crate alloc;

use ds323x::DateTimeAccess;
use weact_studio_epd::{
    graphics::{Display290TriColor, DisplayRotation},
    TriColor, WeActStudio290TriColorDriver,
};
use wifi::{connection_handler_task, net_runner_task};

mod calendar_utils;
mod ntp;
mod wifi;

pub type SpiBusMutex = Mutex<CriticalSectionRawMutex, SpiDmaBus<'static, Async>>;
pub type I2cBusMutex =
    blocking_mutex::Mutex<CriticalSectionRawMutex, RefCell<I2c<'static, Blocking>>>;
pub type Ds323xTypeConcrete = Ds323x<
    I2cInterface<I2cDevice<'static, CriticalSectionRawMutex, I2c<'static, Blocking>>>,
    DS3231,
>;
pub type RtcDs323x = blocking_mutex::Mutex<CriticalSectionRawMutex, RefCell<Ds323xTypeConcrete>>;

static RTC_CLOCK: OnceLock<RtcDs323x> = OnceLock::new();

#[derive(Debug)]
pub enum RtcClockError {
    I2cClockError(<Ds323xTypeConcrete as DateTimeAccess>::Error),
    ClockCellNotSet,
}

/// Get time from the RTC clock on the I2C bus
pub fn get_rtc_time() -> Result<NaiveDateTime, RtcClockError> {
    RTC_CLOCK
        .try_get()
        .ok_or(RtcClockError::ClockCellNotSet)
        .map_err(|e| {
            error!("RTC_CLOCK is not set!");
            e
        })?
        .lock(|rtc_lock| rtc_lock.borrow_mut().datetime())
        .map_err(RtcClockError::I2cClockError)
}

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.init_with(|| $val);
        x
    }};
}

#[main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    esp_alloc::heap_allocator!(72 * 1024);

    let delay = embassy_time::Delay;

    esp_println::logger::init_logger_from_env();

    let timg1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timg1.timer0);

    info!("Embassy initialized!");

    info!("RNG init");

    let mut rng = Rng::new(peripherals.RNG);

    info!("WiFi init");

    let timg0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let wifi_init = mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(timg0.timer0, rng, peripherals.RADIO_CLK).unwrap()
    );

    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&*wifi_init, peripherals.WIFI, WifiStaDevice).unwrap();

    info!("Initializing network stack");

    let net_config = embassy_net::Config::dhcpv4(Default::default());
    let net_seed = (rng.random() as u64) << 32 | rng.random() as u64;

    let (net_stack, net_runner) = embassy_net::new(
        wifi_interface,
        net_config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        net_seed,
    );

    spawner.spawn(connection_handler_task(controller)).ok();
    spawner.spawn(net_runner_task(net_runner)).ok();

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

    // At this point the RTC_CLOCK is not yet initialized, guranteed to be initialized HERE. Any
    // usage must be AFTER this.
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

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let spi_bus = mk_static!(SpiBusMutex, {
        let spi_dma_bus: SpiDmaBus<'static, Async> = Spi::<'static, _>::new(
            peripherals.SPI2,
            Config::default().with_frequency(2_u32.MHz()),
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

    info!("buffer init");

    let mut display = Display290TriColor::new();
    display.set_rotation(DisplayRotation::Rotate90);

    info!("Drawing");

    let style_black = MonoTextStyle::new(&PROFONT_24_POINT, TriColor::Black);
    let style_red = MonoTextStyle::new(&PROFONT_24_POINT, TriColor::Red);
    let _ = Text::with_text_style(
        "Hello, world!",
        Point::new(8, 68),
        style_black,
        TextStyle::default(),
    )
    .draw(&mut display);
    let _ = Text::with_text_style(
        "Hello, world!",
        Point::new(8, 34),
        style_red,
        TextStyle::default(),
    )
    .draw(&mut display);

    info!("Display full update");

    driver.full_update(&display).await.unwrap();

    info!("Pre sleep");

    driver.sleep().await.unwrap();

    info!("Post sleep");

    // TODO: Spawn some tasks
    let _ = spawner;

    loop {
        info!("Hello world!");
        Timer::after_secs(1).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.22.0/examples/src/bin
}
