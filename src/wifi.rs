use std::str::FromStr;
use std::thread;
use std::time::Duration;

use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::sys::EspError;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::*;

pub fn connect_to_wifi<'a>(
    ssid: &'a str,
    password: &'a str,
    modem: impl Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
) -> Result<EspWifi<'a>, EspError> {
    log::info!("Connecting to Wi-Fi network: {}", ssid);
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    log::info!("Creating Wi-Fi instance");
    let mut wifi = EspWifi::new(modem, sysloop, Some(nvs))?;

    log::info!("Setting Wi-Fi configuration");
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: heapless::String::from_str(ssid)
            .map_err(|_| EspError::from(1).expect("Failed to create string"))?,
        password: heapless::String::from_str(password)
            .map_err(|_| EspError::from(1).expect("Failed to create string"))?,
        ..Default::default()
    }))?;

    log::info!("Starting Wi-Fi");

    wifi.start()?;

    wifi.connect()?;

    log::info!("Waiting for Wi-Fi connection");

    while !wifi.is_connected().unwrap_or(false) {
        thread::sleep(Duration::from_secs(1));
    }

    log::info!("Connected to Wi-Fi network: {}", ssid);
    Ok(wifi)
}
