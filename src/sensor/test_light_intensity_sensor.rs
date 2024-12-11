use super::Sensor;
use rand;

pub struct TestLightIntensitySensor {}

impl TestLightIntensitySensor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Sensor for TestLightIntensitySensor {
    fn read(&mut self) -> Vec<f32> {
        // generate random value between 100 and 300
        let value = rand::random::<f32>() * 200.0 + 100.0;
        log::info!("Light intensity: {}%", value);
        vec![value]
    }
}
