use std::rc::Rc;

use embedded_hal::{delay::DelayNs, i2c::I2c};
use esp_idf_hal::{
    delay::{Delay, Ets},
    i2c::{I2cConfig, I2cDriver},
    prelude::Peripherals,
};

use bh1750::BH1750;

use crate::config::AppConfig;

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
            .get_current_measurement(bh1750::Resolution::High)
        {
            Ok(lux) => vec![lux as f32],
            Err(_) => vec![0.0],
        }
    }
}

// pub fn init_sensor(
//     peripherals: Rc<Peripherals>,
//     config: &AppConfig,
// ) -> Result<LightIntensitySensor<impl I2c, impl DelayNs>, String> {
//     let sda = &peripherals.pins.gpio4;
//     let scl = &peripherals.pins.gpio5;
//     let i2c = &peripherals.i2c0;
//     let i2c_config = I2cConfig::default();
//     let i2c_driver = I2cDriver::new(*i2c, *sda, *scl, &i2c_config).map_err(|e| e.to_string())?;

//     let delay: Delay = Default::default();
//     let bh1750 = BH1750::new(i2c_driver, delay, false);
//     Ok(LightIntensitySensor::new(bh1750))
// }
