pub mod soil_humidity_sensor;
pub mod light_intensity_sensor;

pub trait Sensor {
    fn read(&mut self) -> f64;
}
