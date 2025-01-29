#![no_std]
#![no_main]

use core::cell::RefCell;

use display_interface_spi::SPIInterface;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::MonoTextStyle,
    prelude::{Point, Primitive},
    primitives::{Line, PrimitiveStyle},
    text::{Text, TextStyle},
    Drawable,
};
use embedded_hal_bus::spi::{ExclusiveDevice, RefCellDevice};
use esp_backtrace as _;
use esp_hal::{
    dma::{Dma, DmaPriority, DmaRxBuf, DmaTxBuf},
    dma_buffers,
    gpio::{Input, Level, NoPin, Output, Pull},
    peripherals::SPI2,
    prelude::*,
    spi::{
        master::{Config, Spi, SpiDmaBus},
        SpiMode,
    },
    Async, Blocking,
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

    let mut delay = embassy_time::Delay;
    //let mut delay = esp_hal::delay::Delay::new();

    esp_println::logger::init_logger_from_env();

    let timer0 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");

    /*
    let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let _init = esp_wifi::init(
        timer1.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();
    */

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
    //.with_cs(peripherals.GPIO5)
    .with_cs(NoPin)
    .with_miso(NoPin)
    .with_sck(peripherals.GPIO18)
    .with_mosi(peripherals.GPIO21)
    .with_dma(dma_channel.configure(false, DmaPriority::Priority0))
    .with_buffers(dma_rx_buf, dma_tx_buf);
    //let spi_bus = RefCell::new(spi_bus);
    let spi_bus = embassy_sync::mutex::Mutex::<NoopRawMutex, _>::new(spi_bus);

    info!("Initializing spi device");

    let mut spi_device = SpiDevice::new(&spi_bus, cs);
    //let mut spi = RefCellDevice::new(&spi_bus, cs, delay).unwrap();
    //let mut spi_device =
    //    ExclusiveDevice::new(spi_bus, cs, delay).expect("SPI device initialization error");
    let mut spi_interface = SPIInterface::new(spi_device, dc);

    info!("Initializing epd");

    //let mut epd =
    //    Epd2in9bc::new(&mut spi, busy_in, dc, rst, &mut delay, None).expect("EPD creation error");

    let mut driver = WeActStudio290TriColorDriver::new(spi_interface, busy_in, rst, delay);
    driver.init().await.unwrap();

    info!("buffer init");

    let mut display = Display290TriColor::new();
    display.set_rotation(DisplayRotation::Rotate90);

    info!("Drawing");

    let style = MonoTextStyle::new(&PROFONT_24_POINT, TriColor::Black);
    let _ = Text::with_text_style(
        "Hello, world!",
        Point::new(8, 68),
        style,
        TextStyle::default(),
    )
    .draw(&mut display);

    driver.full_update(&display).await.unwrap();

    /*
        let mut mono_display = Display2in9bc::default();
        mono_display.set_rotation(DisplayRotation::Rotate90);

        let _ = Line::new(Point::new(0, 120), Point::new(0, 200))
            .into_styled(PrimitiveStyle::with_stroke(Color::Black, 1))
            .draw(&mut mono_display);

        let mut chromatic_display = Display2in9bc::default();
        chromatic_display.set_rotation(DisplayRotation::Rotate90);

        let _ = Line::new(Point::new(15, 120), Point::new(15, 200))
            .into_styled(PrimitiveStyle::with_stroke(Color::Black, 1))
            .draw(&mut chromatic_display);
    */
    info!("Updating color frame");

    /*
        epd.update_color_frame(
            &mut spi,
            &mut delay,
            mono_display.buffer(),
            chromatic_display.buffer(),
        )
        .unwrap();

        epd.display_frame(&mut spi, &mut delay).unwrap();
    */
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
