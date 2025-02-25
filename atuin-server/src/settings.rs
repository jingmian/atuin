use std::{io::prelude::*, path::PathBuf};

use config::{Config, Environment, File as ConfigFile, FileFormat};
use eyre::{eyre, Result};
use fs_err::{create_dir_all, File};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

static EXAMPLE_CONFIG: &str = include_str!("../server.toml");

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Metrics {
    pub enable: bool,
    pub host: String,
    pub port: u16,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            enable: false,
            host: String::from("127.0.0.1"),
            port: 9001,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings<DbSettings> {
    pub host: String,
    pub port: u16,
    pub path: String,
    pub open_registration: bool,
    pub max_history_length: usize,
    pub max_record_size: usize,
    pub page_size: i64,
    pub register_webhook_url: Option<String>,
    pub register_webhook_username: String,
    pub metrics: Metrics,

    #[serde(flatten)]
    pub db_settings: DbSettings,
}

impl<DbSettings: DeserializeOwned> Settings<DbSettings> {
    pub fn new() -> Result<Self> {
        let mut config_file = if let Ok(p) = std::env::var("ATUIN_CONFIG_DIR") {
            PathBuf::from(p)
        } else {
            let mut config_file = PathBuf::new();
            let config_dir = atuin_common::utils::config_dir();
            config_file.push(config_dir);
            config_file
        };

        config_file.push("server.toml");

        // create the config file if it does not exist
        let mut config_builder = Config::builder()
            .set_default("host", "127.0.0.1")?
            .set_default("port", 8888)?
            .set_default("open_registration", false)?
            .set_default("max_history_length", 8192)?
            .set_default("max_record_size", 1024 * 1024 * 1024)? // pretty chonky
            .set_default("path", "")?
            .set_default("register_webhook_username", "")?
            .set_default("page_size", 1100)?
            .set_default("metrics.enable", false)?
            .set_default("metrics.host", "127.0.0.1")?
            .set_default("metrics.port", 9001)?
            .add_source(
                Environment::with_prefix("atuin")
                    .prefix_separator("_")
                    .separator("__"),
            );

        config_builder = if config_file.exists() {
            config_builder.add_source(ConfigFile::new(
                config_file.to_str().unwrap(),
                FileFormat::Toml,
            ))
        } else {
            create_dir_all(config_file.parent().unwrap())?;
            let mut file = File::create(config_file)?;
            file.write_all(EXAMPLE_CONFIG.as_bytes())?;

            config_builder
        };

        let config = config_builder.build()?;

        config
            .try_deserialize()
            .map_err(|e| eyre!("failed to deserialize: {}", e))
    }
}

pub fn example_config() -> &'static str {
    EXAMPLE_CONFIG
}
