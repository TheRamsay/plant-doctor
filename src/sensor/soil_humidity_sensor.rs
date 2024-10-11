use std::borrow::Borrow;

use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::{oneshot::AdcChannelDriver, AdcContDriver};
use esp_idf_hal::adc::{oneshot::*, Adc, ADC1};
use esp_idf_hal::gpio::ADCPin;

use super::Sensor;

pub struct SoilHumiditySensor<'a, A, P>
where
    A: Borrow<AdcDriver<'a, P::Adc>>,
    P: ADCPin,
{
    adc: A,
    adc_pin: AdcChannelDriver<'a, P, A>,
    wet_value: i16,
    dry_value: i16,
}

impl<'a, A, P> SoilHumiditySensor<'a, A, P>
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

impl<'a, A, P> Sensor for SoilHumiditySensor<'a, A, P>
where
    A: Borrow<AdcDriver<'a, P::Adc>>,
    P: ADCPin,
{
    fn read(&mut self) -> f64 {
        let adc_value = self.adc.borrow().read(&mut self.adc_pin).unwrap() as i16;
        ((self.dry_value - adc_value) as f64 / (self.dry_value - self.wet_value) as f64) * 100.0
    }
}
