pub mod air_sensor;
pub mod light_intensity_sensor;
pub mod soil_humidity_sensor;
pub mod test_light_intensity_sensor;
pub mod test_soil_moisture_sensor;

pub trait Sensor {
    fn read(&mut self) -> Vec<f32>;
}

pub enum SensorType {
    AirSensor,
    LightIntensitySensor,
    SoilHumiditySensor,
}
