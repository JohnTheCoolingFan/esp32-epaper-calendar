#![no_std]
#![no_main]

use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_net::{Runner, StackResources};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
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
    rng::Rng,
    spi::master::{Config, Spi, SpiDmaBus},
    Async,
};
use esp_hal_embassy::main;
use esp_wifi::{
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
    EspWifiController,
};
use heapless::String;
use log::{debug, error, info, trace, warn};

extern crate alloc;

use profont::PROFONT_24_POINT;
use static_cell::StaticCell;
use weact_studio_epd::{
    graphics::{Display290TriColor, DisplayRotation},
    TriColor, WeActStudio290TriColorDriver,
};

mod calendar;

const SSID: &'static str = env!("SSID");
const WIFI_PASSWORD: &'static str = env!("WIFI_PASSWORD");

pub type SpiBusMutex = Mutex<CriticalSectionRawMutex, SpiDmaBus<'static, Async>>;

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
        esp_wifi::init(timg0.timer0, rng.clone(), peripherals.RADIO_CLK).unwrap()
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

    info!("Initializing pins");

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
        let spi_dma_bus: SpiDmaBus<'static, Async> =
            Spi::<'static, _>::new(peripherals.SPI2, Config::default())
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
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.22.0/examples/src/bin
}

#[embassy_executor::task]
async fn connection_handler_task(mut controller: WifiController<'static>) {
    info!("Starting wifi connection handler task");
    info!("Devcie capabilities: {:?}", controller.capabilities());
    loop {
        match esp_wifi::wifi::wifi_state() {
            WifiState::StaConnected => {
                controller.wait_for_event(WifiEvent::StaDisconnected).await;
                Timer::after(Duration::from_secs(5)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.try_into().unwrap(),
                password: WIFI_PASSWORD.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            info!("Starting wifi");
            controller.start_async().await.unwrap();
            info!("Wifi started");
        }
        info!("About to connect");

        match controller.connect_async().await {
            Ok(_) => info!("Wifi connected"),
            Err(e) => {
                error!("Faield to connect to wifi: {e:?}");
                Timer::after(Duration::from_secs(1)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_runner_task(mut runner: Runner<'static, WifiDevice<'static, WifiStaDevice>>) {
    runner.run().await
}
