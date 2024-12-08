use std::error::Error;

use serde::Deserialize;

const CONFIG_FILE: &str = "config.toml";

#[derive(Deserialize)]
pub struct AppConfig {
    pub wifi: WifiConfig,
    pub home_assistant: HomeAssistantConfig,
    pub gpio_pins: GpioPinsConfig,
}

#[derive(Deserialize)]
pub struct WifiConfig {
    pub ssid: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct HomeAssistantConfig {
    pub url: String,
}

#[derive(Deserialize)]
pub struct GpioPinsConfig {
    pub bh1750_sda: u8,
    pub bh1750_scl: u8,
    pub dht22: u8,
    pub moisture_adc: u8,
}

pub fn load_config() -> Result<AppConfig, String> {
    Ok(AppConfig {
        wifi: WifiConfig {
            ssid: "Vodafone-94BC".to_string(),
            password: "HDd7DtMbUJMHL4tU".to_string(),
        },
        home_assistant: HomeAssistantConfig {
            url: "mqtt://192.168.0.83:1883".into(),
        },
        gpio_pins: GpioPinsConfig {
            dht22: 4,
            bh1750_sda: 5,
            bh1750_scl: 6,
            moisture_adc: 34,
        },
    })
}
