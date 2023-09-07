use std::{env, sync::OnceLock};

use crate::{Error, Result};

pub fn config() -> &'static Config {
    // 使用OnceLock,避免多次初始化
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env()
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"))
    })
}

#[allow(non_snake_case)]
pub struct Config {
    pub WEB_FOLDER: String,
}

impl Config {
    fn load_from_env() -> Result<Self> {
        Ok(Self {
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}
