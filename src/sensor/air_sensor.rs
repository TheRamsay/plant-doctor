use embedded_dht_rs::dht22::Dht22;
use embedded_hal::{
    delay::DelayNs,
    digital::{InputPin, OutputPin},
};
use esp_idf_hal::{
    delay::Delay,
    gpio::{AnyIOPin, Input, Output, PinDriver},
    prelude::Peripherals,
};

use super::Sensor;

pub struct AirSensor<PIN: InputPin + OutputPin, DELAY: DelayNs> {
    dht22: Dht22<PIN, DELAY>,
}

impl<PIN, DELAY> AirSensor<PIN, DELAY>
where
    PIN: InputPin + OutputPin,
    DELAY: DelayNs,
{
    pub fn new(dht22: Dht22<PIN, DELAY>) -> Self {
        Self { dht22 }
    }
}

impl<PIN, DELAY> Sensor for AirSensor<PIN, DELAY>
where
    PIN: InputPin + OutputPin,
    DELAY: DelayNs,
{
    fn read(&mut self) -> Vec<f32> {
        log::info!("AHOJDA");
        match self.dht22.read() {
            Ok(result) => vec![result.temperature as f32, result.humidity as f32],
            Err(e) => {
                log::error!("Failed to read DHT22: {:?}", e);
                return vec![];
            }
        }
    }
}
