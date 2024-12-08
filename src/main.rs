use config::load_config;
use epd_waveshare::epd2in9_v2::{Display2in9, Epd2in9};
use epd_waveshare::prelude::WaveshareDisplay;
use std::rc::Rc;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use wifi::connect_to_wifi;

use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::*;
use esp_idf_hal::delay::{Delay, Ets};
use esp_idf_hal::gpio::{AnyIOPin, Gpio2, PinDriver};
use esp_idf_hal::i2c::{I2c, I2cConfig, I2cDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::SpiDriverConfig;
use esp_idf_hal::spi::{config::Config, SpiDeviceDriver};

use bh1750::BH1750;
use esp_idf_hal::sys::EspError;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS};
use esp_idf_svc::nvs::{EspDefaultNvs, EspDefaultNvsPartition};
use esp_idf_svc::ping::EspPing;
use esp_idf_svc::wifi::*;
use esp_idf_svc::{ipv4, netif::*};
use image::{image_to_binary, load_and_process_image};
use plant_display::PlantDisplay;
use publisher::sensor_config::SensorConfig;
use sensor::{light_intensity_sensor, SensorType};

const WET_VALUE: i16 = 900;
const DRY_VALUE: i16 = 2500;
mod board;
mod config;
mod image;
mod plant_display;
mod publisher;
mod sensor;
mod wifi;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    let app_config = load_config().expect("Failed to load configuration");
    log::info!("Loaded config!");

    let peripherals = Peripherals::take().unwrap();

    let adc = Rc::new(AdcDriver::new(peripherals.adc1).unwrap());
    let config = AdcChannelConfig {
        attenuation: DB_11,
        calibration: true,
        ..Default::default()
    };
    let adc_pin = AdcChannelDriver::new(adc.clone(), peripherals.pins.gpio34, &config).unwrap();
    let humidity_sensor = sensor::soil_humidity_sensor::SoilMoistureSensor::new(
        adc.clone(),
        adc_pin,
        WET_VALUE,
        DRY_VALUE,
    );

    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio22;
    let i2c = peripherals.i2c0;
    let i2c_config = I2cConfig::default();
    let i2c_driver = I2cDriver::new(i2c, sda, scl, &i2c_config).expect("Failed to initialize I2C");

    let delay: Delay = Default::default();
    delay.delay_ms(200);
    let bh1750 = BH1750::new(i2c_driver, delay, true);
    let light_sensor = sensor::light_intensity_sensor::LightIntensitySensor::new(bh1750);

    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio18;
    let serial_out = peripherals.pins.gpio23;
    let cs = PinDriver::output(peripherals.pins.gpio15).unwrap();
    let busy_in = PinDriver::input(peripherals.pins.gpio17).unwrap();
    let dc = PinDriver::output(peripherals.pins.gpio16).unwrap();
    let rst = PinDriver::output(peripherals.pins.gpio4).unwrap();

    let config = Config::new().baudrate(112500.into());
    let mut device = SpiDeviceDriver::new_single(
        spi,
        sclk,
        serial_out,
        Option::<Gpio2>::None,
        Option::<AnyIOPin>::None,
        &SpiDriverConfig::default(),
        &config,
    )
    .unwrap();

    let mut delay: Delay = Default::default();
    let mut epd = Epd2in9::new(&mut device, busy_in, dc, rst, &mut delay, None).unwrap();

    log::info!("Initializing display");

    let mut display = Display2in9::default();

    let mut plant_display = PlantDisplay::new(epd, display, delay, device);

    let mut display_input = plant_display::DisplayInput {
        plant_name: "Aloe Vera :3".into(),
        soil_moisture: 50.0,
        light_intensity: 343.1,
    };

    plant_display.display_input(&display_input);

    log::info!("Display initialized");

    pub type SensorItem = (Box<dyn sensor::Sensor>, SensorConfig, SensorType);
    let mut sensors: Vec<SensorItem> = vec![
        (
            Box::new(light_sensor),
            SensorConfig {
                topic: "sensor/light_intensity".into(),
            },
            SensorType::LightIntensitySensor,
        ),
        (
            Box::new(humidity_sensor),
            SensorConfig {
                topic: "sensor/soil_moisture".into(),
            },
            SensorType::SoilHumiditySensor,
        ),
    ];

    log::info!("Connecting to Wi-Fi network");

    let wifi = connect_to_wifi(
        &app_config.wifi.ssid,
        &app_config.wifi.password,
        peripherals.modem,
    )
    .expect("Failed to connect to Wi-Fi");

    log::info!("Connected to Wi-Fi network, starting MQTT client");

    let mut ping_configuration = EspPing::new(0);
    match ping_configuration.ping(
        ipv4::Ipv4Addr::from_str("192.168.0.83").unwrap(),
        &esp_idf_svc::ping::Configuration::default(),
    ) {
        Ok(summary) => log::info!("Ping successful with summary: {:?}", summary),
        Err(e) => log::error!("Ping failed: {:?}", e),
    }

    let mut mqtt_client = EspMqttClient::new_cb(
        &app_config.home_assistant.url,
        &MqttClientConfiguration {
            network_timeout: Duration::from_secs(5),
            ..Default::default()
        },
        |event| {
            log::info!("MQTT Event: {:?}", event.payload());
        },
    )
    .unwrap();

    log::info!("Starting sensor loop");

    loop {
        for (sensor, config, sensor_type) in sensors.iter_mut() {
            log::info!("Reading sensor values");
            let value = sensor
                .read()
                .into_iter()
                .nth(0)
                .unwrap_or_else(f32::default);

            match sensor_type {
                SensorType::LightIntensitySensor => {
                    display_input.light_intensity = value;
                }
                SensorType::SoilHumiditySensor => {
                    display_input.soil_moisture = value;
                }
                _ => {}
            }

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

        plant_display.display_input(&display_input);
        thread::sleep(Duration::from_millis(2000));
    }
}
