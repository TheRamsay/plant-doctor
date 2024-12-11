use bh1750::BH1750;
use embedded_hal::{delay::DelayNs, i2c::I2c};

use super::Sensor;

pub struct LightIntensitySensor<I2C: I2c, DELAY: DelayNs> {
    bh1750: BH1750<I2C, DELAY>,
}

impl<I2C, DELAY> LightIntensitySensor<I2C, DELAY>
where
    I2C: I2c,
    DELAY: DelayNs,
{
    pub fn new(bh1750: BH1750<I2C, DELAY>) -> Self {
        Self { bh1750 }
    }
}

impl<I2C, DELAY> Sensor for LightIntensitySensor<I2C, DELAY>
where
    I2C: I2c,
    DELAY: DelayNs,
{
    fn read(&mut self) -> Vec<f32> {
        // Attempt to read light intensity
        match self
            .bh1750
            .get_one_time_measurement(bh1750::Resolution::High)
        {
            Ok(lux) => {
                log::info!("Light intensity: {} lux", lux);
                vec![lux as f32]
            }
            Err(e) => {
                log::error!("Error reading light intensity: {:?}", e);
                vec![0.0]
            }
        }
    }
}
