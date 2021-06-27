use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use once_cell::sync::OnceCell;
use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use toml;

use crate::ServerError;


pub(crate) static CONFIG_PATH: OnceCell<PathBuf> = OnceCell::new();
pub(crate) static CONFIG: OnceCell<RwLock<Config>> = OnceCell::new();


#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct Config {
    pub db_conn_string: String,
    pub http_listen: String,
    pub auth_tokens: Vec<String>,
    pub base_url: String,
}

pub(crate) async fn load_config() -> Result<(), ServerError> {
    let path = CONFIG_PATH
        .get().expect("configuration path missing");

    let config: Config = {
        let mut config_file = File::open(path)
            .map_err(|e| ServerError::OpeningConfigFile(e))?;
        let mut config_str = String::new();
        config_file.read_to_string(&mut config_str)
            .map_err(|e| ServerError::ReadingConfigFile(e))?;
        toml::from_str(&config_str)
            .map_err(|e| ServerError::ParsingConfigFile(e))?
    };

    match CONFIG.get() {
        Some(cg) => {
            let mut config_guard = cg
                .write().await;
            *config_guard = config;
        },
        None => {
            CONFIG
                .set(RwLock::new(config)).expect("failed to set lock");
        },
    }

    Ok(())
}
