use super::Sensor;

pub struct LightIntensitySensor();

impl LightIntensitySensor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Sensor for LightIntensitySensor {
    fn read(&mut self) -> f64 {
        0.0
    }
}
