use embassy_net::Runner;
use embassy_time::Timer;
use esp_wifi::wifi::{
    ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
    WifiState,
};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

const SSID: &str = env!("SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

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
