#![no_std]
#![no_main]

use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::MonoTextStyle,
    prelude::*,
    text::{Text, TextStyle},
    Drawable,
};
use esp_backtrace as _;
use esp_hal::{
    dma::{Dma, DmaPriority, DmaRxBuf, DmaTxBuf},
    dma_buffers,
    gpio::{Input, Level, NoPin, Output, Pull},
    prelude::*,
    spi::{
        master::{Config, Spi, SpiDmaBus},
        SpiMode,
    },
    Async,
};
use log::info;

extern crate alloc;

use profont::PROFONT_24_POINT;
//use epd_waveshare::{epd2in9bc::*, prelude::*};
use weact_studio_epd::{
    graphics::{Display290TriColor, DisplayRotation},
    TriColor, WeActStudio290TriColorDriver,
};

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

    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");
    info!("WiFi init");

    let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let _init = esp_wifi::init(
        timer1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    info!("Initializing pins");

    let cs = Output::new_typed(peripherals.GPIO5, Level::High);
    let busy_in = Input::new_typed(peripherals.GPIO4, Pull::Up);
    let rst = Output::new_typed(peripherals.GPIO10, Level::High);
    let dc = Output::new_typed(peripherals.GPIO17, Level::Low);

    info!("Initializing spi bus");

    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(32000);
    let dma_rx_buf = DmaRxBuf::new(rx_descriptors, rx_buffer).unwrap();
    let dma_tx_buf = DmaTxBuf::new(tx_descriptors, tx_buffer).unwrap();

    let spi_bus: SpiDmaBus<'static, Async, _> = Spi::<'static, Async, _>::new_typed_with_config(
        peripherals.SPI2,
        Config {
            frequency: 100.kHz(),
            mode: SpiMode::Mode0,
            ..Config::default()
        },
    )
    .with_cs(NoPin)
    .with_miso(NoPin)
    .with_sck(peripherals.GPIO18)
    .with_mosi(peripherals.GPIO21)
    .with_dma(dma_channel.configure(false, DmaPriority::Priority0))
    .with_buffers(dma_rx_buf, dma_tx_buf);
    let spi_bus = embassy_sync::mutex::Mutex::<NoopRawMutex, _>::new(spi_bus);

    info!("Initializing spi device");

    let spi_device = SpiDevice::new(&spi_bus, cs);
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
