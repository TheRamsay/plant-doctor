use super::Sensor;
use rand;

pub struct TestSoilMoistureSensor {}

impl TestSoilMoistureSensor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Sensor for TestSoilMoistureSensor {
    fn read(&mut self) -> Vec<f32> {
        // generate random value between 0 and 100
        let value = rand::random::<f32>() * 100.0;
        log::info!("Soil moisture: {}%", value);
        vec![value]
    }
}
