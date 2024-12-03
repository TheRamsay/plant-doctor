use embedded_hal::delay::DelayNs;
use esp_idf_hal::{delay::Ets, i2c::I2c};

use crate::driver::bh1750::BH1750;

use super::Sensor;

pub struct LightIntensitySensor<I2C> {
    bh1750: BH1750<I2C>,
}

impl<I2C> LightIntensitySensor<I2C>
where
    I2C: I2c,
{
    pub fn new(bh1750: BH1750<I2C>) -> Self {
        Self { bh1750 }
    }
}

impl<I2C> Sensor for LightIntensitySensor<I2C> {
    fn read(&mut self) -> f64 {
        let delay = Ets;

        let x = &self.bh1750;
        // x.

        1 as f64
    }
}
