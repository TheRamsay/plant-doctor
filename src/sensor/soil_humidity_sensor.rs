use std::borrow::Borrow;
use std::rc::Rc;

use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::AdcChannelDriver;
use esp_idf_hal::adc::{oneshot::*, ADC1};
use esp_idf_hal::gpio::{ADCPin, Gpio34, Pin};
use esp_idf_hal::prelude::Peripherals;

use crate::config::AppConfig;

use super::Sensor;

pub struct SoilMoistureSensor<'a, A, P>
where
    A: Borrow<AdcDriver<'a, P::Adc>>,
    P: ADCPin,
{
    adc: A,
    adc_pin: AdcChannelDriver<'a, P, A>,
    wet_value: i16,
    dry_value: i16,
}

impl<'a, A, P> SoilMoistureSensor<'a, A, P>
where
    A: Borrow<AdcDriver<'a, P::Adc>>,
    P: ADCPin,
{
    pub fn new(
        adc: A,
        adc_pin: AdcChannelDriver<'a, P, A>,
        wet_value: i16,
        dry_value: i16,
    ) -> Self {
        Self {
            adc,
            adc_pin,
            wet_value,
            dry_value,
        }
    }
}

impl<'a, A, P> Sensor for SoilMoistureSensor<'a, A, P>
where
    A: Borrow<AdcDriver<'a, P::Adc>>,
    P: ADCPin,
{
    fn read(&mut self) -> Vec<f32> {
        let adc_value = self.adc.borrow().read(&mut self.adc_pin).unwrap() as i16;
        log::info!("ADC Value: {}", adc_value);
        let percentage = ((self.dry_value - adc_value) as f32
            / (self.dry_value - self.wet_value) as f32)
            * 100.0;

        // cap the percentage to 0-100
        let percentage = percentage.max(0.0).min(100.0);

        vec![percentage]
    }
}

pub fn init_sensor<'a, P, A>(
    peripherals: Peripherals,
    app_config: AppConfig,
) -> Result<SoilMoistureSensor<'a, Rc<AdcDriver<'a, ADC1>>, Gpio34>, String>
where
    P: ADCPin<Adc = ADC1> + Pin,
{
    let adc = Rc::new(AdcDriver::new(peripherals.adc1).unwrap());

    let config = AdcChannelConfig {
        attenuation: DB_11,
        calibration: true,
        ..Default::default()
    };

    let adc_pin = AdcChannelDriver::new(adc.clone(), peripherals.pins.gpio34, &config).unwrap();

    let humidity_sensor = SoilMoistureSensor::new(adc, adc_pin, 2000, 10000);

    Ok(humidity_sensor)
}
