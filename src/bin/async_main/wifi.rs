use embassy_executor::Spawner;
use embassy_net::{
    DhcpConfig, Runner, Stack, StackResources,
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
};
use embassy_time::Timer;
use esp_hal::{
    peripherals::{RADIO_CLK, TIMG0, WIFI},
    rng::Rng,
    timer::timg::TimerGroup,
};
use esp_wifi::{
    EspWifiController,
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

#[cfg(feature = "isdayoff")]
use crate::isdayoff::HttpClientConcrete;

const SSID: &str = env!("SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.init_with(|| $val);
        x
    }};
}

pub fn init_wifi(
    spawner: &Spawner,
    timg0: TimerGroup<TIMG0>,
    rng: Rng,
    radio_clk: RADIO_CLK,
    per_wifi: WIFI,
) -> WifiDevice<'static, WifiStaDevice> {
    info!("WiFi init");

    let wifi_init = mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(timg0.timer0, rng, radio_clk).unwrap()
    );

    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&*wifi_init, per_wifi, WifiStaDevice).unwrap();

    spawner.spawn(connection_handler_task(controller)).ok();

    wifi_interface
}

pub fn init_networking(
    spawner: &Spawner,
    mut rng: Rng,
    wifi_interface: WifiDevice<'static, WifiStaDevice>,
) -> Stack<'static> {
    info!("Initializing network stack");

    let net_config = embassy_net::Config::dhcpv4({
        let mut config = DhcpConfig::default();
        config.hostname = Some("ESP32-Epaper-Calendar".try_into().unwrap());
        config
    });
    let net_seed = ((rng.random() as u64) << 32) | rng.random() as u64;

    let (net_stack, net_runner) = embassy_net::new(
        wifi_interface,
        net_config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        net_seed,
    );

    spawner.spawn(net_runner_task(net_runner)).ok();

    net_stack
}

#[cfg(feature = "isdayoff")]
pub fn init_tcp_http(net_stack: Stack<'static>) -> &'static mut HttpClientConcrete {
    info!("TCP/HTTP Client init");
    let tcp_state =
        mk_static!(TcpClientState<1, 4096, 4096>, {TcpClientState::<1, 4096, 4096>::new()});
    let tcp_client = mk_static!(TcpClient<1, 4096, 4096>, {
        TcpClient::new(net_stack, &*tcp_state)
    });
    let dns_socket = mk_static!(DnsSocket<'static>, DnsSocket::new(net_stack));

    let http_client = mk_static!(HttpClientConcrete, {
        reqwless::client::HttpClient::new(&*tcp_client, &*dns_socket)
    });

    http_client
}

#[embassy_executor::task]
pub async fn connection_handler_task(mut controller: WifiController<'static>) {
    info!("Starting wifi connection handler task");
    info!("Device capabilities: {:?}", controller.capabilities());
    loop {
        if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after_secs(5).await
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
                Timer::after_secs(1).await
            }
        }
    }
}

#[embassy_executor::task]
pub async fn net_runner_task(mut runner: Runner<'static, WifiDevice<'static, WifiStaDevice>>) {
    runner.run().await
}
