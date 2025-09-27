use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub ip: String,
    pub port: u16,

    pub data_path: String,

    //===
    pub explorer_api_url: String,
}

impl ApiConfig {
    pub fn parse_from_file() -> Self {
        let config_raw = std::fs::read("./config.toml")
            .expect("Failed to open config.toml file (file not exist?)");
        let config_str = String::from_utf8(config_raw)
            .expect("Fail to parse config to string");

        toml::from_str::<ApiConfig>(&config_str)
            .expect("Fail to deserialize config file to ApiConfig")
    }

}
