use serde::Deserialize;

const CONFIG_FILE: &str = "config.toml";

#[derive(Deserialize)]
pub struct AppConfig {
    pub wifi: WifiConfig,
    pub home_assistant: HomeAssistantConfig,
    pub plant_display: PlantDisplayConfig,
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
pub struct PlantDisplayConfig {
    pub plant_name: String,
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
        plant_display: PlantDisplayConfig {
            plant_name: "Vilem zahradni".into(),
        },
    })
}
