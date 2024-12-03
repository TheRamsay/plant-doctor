use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// use bh1750::BH1750;

use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::*;
use esp_idf_hal::i2c::I2c;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::peripherals::Peripherals;

use esp_idf_hal::sys::EspError;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS};
use esp_idf_svc::nvs::{EspDefaultNvs, EspDefaultNvsPartition};
use esp_idf_svc::ping::EspPing;
use esp_idf_svc::wifi::*;
use esp_idf_svc::{ipv4, netif::*};
use publisher::sensor_config::SensorConfig;

use driver::bh1750::BH1750;

const WET_VALUE: i16 = 950;
const DRY_VALUE: i16 = 2500;

mod driver;
mod publisher;
mod sensor;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let peripherals = Peripherals::take().unwrap();

    let adc = Rc::new(AdcDriver::new(peripherals.adc1).unwrap());

    let config = AdcChannelConfig {
        attenuation: DB_11,
        calibration: true,
        ..Default::default()
    };

    let adc_pin = AdcChannelDriver::new(adc.clone(), peripherals.pins.gpio34, &config).unwrap();

    let humity_sensor =
        sensor::soil_humidity_sensor::SoilMoistureSensor::new(adc.clone(), adc_pin, 2000, 10000);

    use esp_idf_hal::i2c;

    let i2c = peripherals.i2c0;

    // let bh1750 = BH1750::new(i2c);
    // let bh1750 = BH1750::new(peripherals.i2c0, delay, false);
    // let light_sensor = sensor::light_intensity_sensor::LightIntensitySensor::new(bh1750);

    pub type SensorItem = (Box<dyn sensor::Sensor>, SensorConfig);
    let mut sensors: Vec<SensorItem> = vec![
        (
            Box::new(humity_sensor),
            SensorConfig {
                topic: "sensor/soil_moisture".into(),
            },
        ),
        // (
        //     Box::new(light_sensor),
        //     SensorConfig {
        //         topic: "sensor/light_intensity".into(),
        //     },
        // ),
    ];

    let wifi = connect_to_wifi("Vodafone-94BC", "HDd7DtMbUJMHL4tU", peripherals.modem).unwrap();

    let mut ping_configuration = EspPing::new(0);
    match ping_configuration.ping(
        ipv4::Ipv4Addr::from_str("192.168.0.83").unwrap(),
        &esp_idf_svc::ping::Configuration::default(),
    ) {
        Ok(summary) => log::info!("Ping successful with summary: {:?}", summary),
        Err(e) => log::error!("Ping failed: {:?}", e),
    }

    let mut mqtt_client = EspMqttClient::new_cb(
        "mqtt://192.168.0.83:1883",
        &MqttClientConfiguration {
            network_timeout: Duration::from_secs(5),
            ..Default::default()
        },
        |event| {
            log::info!("MQTT Event: {:?}", event.payload());
        },
    )
    .unwrap();

    loop {
        thread::sleep(Duration::from_millis(2000));

        for (sensor, config) in sensors.iter_mut() {
            let value = sensor.read();
            log::info!("Sensor value: {}", value);

            match mqtt_client.publish(
                config.topic.as_str(),
                QoS::AtMostOnce,
                false,
                format!("{{\"value\": {}}}", value).as_bytes(),
            ) {
                Ok(id) => log::info!("Published message with id {}", id),
                Err(e) => log::error!("Error publishing: {:?}", e),
            };
        }
    }
}

fn connect_to_wifi<'a>(
    ssid: &'a str,
    password: &'a str,
    modem: impl Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
) -> Result<EspWifi<'a>, EspError> {
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = EspWifi::new(modem, sysloop, Some(nvs))?;

    // TODO: fix this magic 😭😭
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: heapless::String::from_str(ssid).map_err(|_| EspError::from(1).unwrap())?,
        password: heapless::String::from_str(password).map_err(|_| EspError::from(1).unwrap())?,
        ..Default::default()
    }))?;

    wifi.start()?;

    wifi.connect()?;

    while !wifi.is_connected().unwrap_or(false) {
        thread::sleep(Duration::from_secs(1));
    }

    log::info!("Connected to Wi-Fi network: {}", ssid);
    Ok(wifi)
}
