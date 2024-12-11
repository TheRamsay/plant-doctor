use bh1750::BH1750;
use config::{load_config, AppConfig};
// use driver::bh1750::BH1750;
use embedded_hal::delay::DelayNs;
use embedded_hal::spi::SpiDevice;
use epd_waveshare::epd2in9_v2::{Display2in9, Epd2in9};
use epd_waveshare::prelude::WaveshareDisplay;
use esp_idf_hal::units::Hertz;
use sensor::light_intensity_sensor::LightIntensitySensor;
use sensor::soil_humidity_sensor::SoilMoistureSensor;
use std::rc::Rc;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use wifi::connect_to_wifi;

use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::{oneshot::*, Adc};
use esp_idf_hal::delay::Delay;
use esp_idf_hal::gpio::{ADCPin, AnyIOPin, Gpio2, InputPin, OutputPin, PinDriver};
use esp_idf_hal::i2c::{I2c, I2cConfig, I2cDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::{config::Config, SpiDeviceDriver};
use esp_idf_hal::spi::{SpiAnyPins, SpiDriverConfig};

use esp_idf_hal::sys::EspError;
use esp_idf_svc::ipv4;
use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS};
use esp_idf_svc::ping::EspPing;
use plant_display::{DisplayInput, PlantDisplay};
use publisher::sensor_config::SensorConfig;
use sensor::{test_light_intensity_sensor, test_soil_moisture_sensor, SensorType};

const WET_VALUE: i16 = 900;
const DRY_VALUE: i16 = 2500;

mod config;
// mod driver;
mod image;
mod plant_display;
mod publisher;
mod sensor;
mod wifi;

pub type SensorItem = (Box<dyn sensor::Sensor>, SensorConfig, SensorType);

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let app_config = load_config().expect("Failed to load configuration");
    log::info!("Loaded config!");

    let peripherals = Peripherals::take().unwrap();

    let humidity_sensor = init_soil_humidity_sensor(
        peripherals.adc1,
        peripherals.pins.gpio34,
        WET_VALUE,
        DRY_VALUE,
    )
    .expect("Failed to initialize soil humidity sensor");

    let light_sensor = init_light_sensor(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
    )
    .unwrap();

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

    let test_light_sensor = test_light_intensity_sensor::TestLightIntensitySensor::new();
    let test_soil_moisture = test_soil_moisture_sensor::TestSoilMoistureSensor::new();

    let sensors: Vec<SensorItem> = vec![
        (
            // Box::new(light_sensor),
            Box::new(test_light_sensor),
            SensorConfig {
                topic: "sensor/light_intensity".into(),
            },
            SensorType::LightIntensitySensor,
        ),
        (
            // Box::new(humidity_sensor),
            Box::new(test_soil_moisture),
            SensorConfig {
                topic: "sensor/soil_moisture".into(),
            },
            SensorType::SoilHumiditySensor,
        ),
    ];

    let wifi = connect_to_wifi(
        &app_config.wifi.ssid,
        &app_config.wifi.password,
        peripherals.modem,
    )
    .expect("Failed to connect to Wi-Fi");

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

    log::info!("Connected to MQTT broker");

    log::info!("Publishing plant name to MQTT");
    mqtt_client
        .publish(
            "config/plant_name",
            QoS::AtMostOnce,
            false,
            app_config.plant_display.plant_name.as_bytes(),
        )
        .unwrap();

    log::info!("Starting sensor loop");

    run_sensor_loop(mqtt_client, plant_display, sensors, &app_config);
}

fn run_sensor_loop(
    mut mqtt_client: EspMqttClient<'static>,
    mut plant_display: PlantDisplay<
        impl SpiDevice,
        impl embedded_hal::digital::InputPin,
        impl embedded_hal::digital::OutputPin,
        impl embedded_hal::digital::OutputPin,
        impl DelayNs,
    >,
    mut sensors: Vec<SensorItem>,
    app_config: &AppConfig,
) {
    loop {
        let mut display_input = DisplayInput {
            plant_name: app_config.plant_display.plant_name.clone(),
            soil_moisture: 0.0,
            light_intensity: 0.0,
        };

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
        thread::sleep(Duration::from_millis(500));
    }
}

fn init_light_sensor(
    i2c: impl Peripheral<P = impl I2c> + 'static,
    sda: impl Peripheral<P = impl InputPin + OutputPin> + 'static,
    scl: impl Peripheral<P = impl InputPin + OutputPin> + 'static,
) -> Result<LightIntensitySensor<impl embedded_hal::i2c::I2c, impl DelayNs>, String> {
    let i2c_config = I2cConfig::new().baudrate(400_000.into());

    let i2c_driver = I2cDriver::new(i2c, sda, scl, &i2c_config).map_err(|e| e.to_string())?;

    let delay: Delay = Default::default();
    let bh1750 = BH1750::new(i2c_driver, delay, false);
    let sensor = LightIntensitySensor::new(bh1750);

    Ok(sensor)
}

fn init_display(
    spi: impl Peripheral<P = impl SpiAnyPins> + 'static,
    sclk: impl Peripheral<P = impl OutputPin> + 'static,
    serial_out: impl Peripheral<P = impl OutputPin> + 'static,
    cs: impl Peripheral<P = impl OutputPin> + 'static,
    busy_in: impl Peripheral<P = impl InputPin> + 'static,
    dc: impl Peripheral<P = impl OutputPin> + 'static,
    rst: impl Peripheral<P = impl OutputPin> + 'static,
) -> Result<
    PlantDisplay<
        impl SpiDevice,
        impl embedded_hal::digital::InputPin,
        impl embedded_hal::digital::OutputPin,
        impl embedded_hal::digital::OutputPin,
        impl DelayNs,
    >,
    EspError,
> {
    let config = Config::new().baudrate(112500.into());
    let mut device = SpiDeviceDriver::new_single(
        spi,
        sclk,
        serial_out,
        Option::<Gpio2>::None,
        Option::<AnyIOPin>::None,
        &SpiDriverConfig::default(),
        &config,
    )?;

    let _ = PinDriver::output(cs)?;
    let busy_in = PinDriver::input(busy_in)?;
    let dc = PinDriver::output(dc)?;
    let rst = PinDriver::output(rst)?;

    let mut delay: Delay = Default::default();
    let epd = Epd2in9::new(&mut device, busy_in, dc, rst, &mut delay, None).unwrap();
    let display = Display2in9::default();
    let plant_display = PlantDisplay::new(epd, display, delay, device);

    Ok(plant_display)
}

fn init_soil_humidity_sensor<
    A: Adc + 'static,
    P: Peripheral<P = A> + 'static,
    Pin: ADCPin<Adc = A>,
>(
    adc: P,
    adc_pin: Pin,
    wet_value: i16,
    dry_value: i16,
) -> Result<SoilMoistureSensor<'static, Rc<AdcDriver<'static, A>>, Pin>, String> {
    let adc =
        Rc::new(AdcDriver::new(adc).map_err(|e| format!("Failed to initialize ADC: {:?}", e))?);
    let config = AdcChannelConfig {
        attenuation: DB_11,
        calibration: true,
        ..Default::default()
    };

    let adc_pin = AdcChannelDriver::new(Rc::clone(&adc), adc_pin, &config)
        .map_err(|e| format!("Failed to initialize ADC pin: {:?}", e))?;

    let humidity_sensor = SoilMoistureSensor::new(adc, adc_pin, wet_value, dry_value);
    Ok(humidity_sensor)
}
