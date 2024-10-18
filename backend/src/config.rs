use std::env;
use std::path::PathBuf;

use tracing::info;

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::from_env().expect("Failed to load config from environment");
}

pub struct Config {
    pub domain: String,
    pub port: u16,
    pub fm_db_path: PathBuf,
    pub pg_db: String,
    pub mnemonic: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenv::dotenv().ok();

        let config = Self {
            domain: env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("Invalid port"),
            fm_db_path: env::var("FM_DB_PATH")?.parse().expect("Invalid FM_DB_PATH"),
            pg_db: env::var("DATABASE_URL")?,
            mnemonic: env::var("MULTIMINT_MNEMONIC_ENV")?,
        };

        info!("Loaded config");
        Ok(config)
    }
}
