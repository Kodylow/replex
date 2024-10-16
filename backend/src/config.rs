use std::env;
use std::path::PathBuf;
use std::str::FromStr;

use fedimint_client::derivable_secret::DerivableSecret;
use fedimint_client::secret::{PlainRootSecretStrategy, RootSecretStrategy};
use fedimint_core::api::InviteCode;
use nostr::hashes::hex::FromHex;
use tracing::info;

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::from_env().expect("Failed to load config from environment");
}

pub struct Config {
    pub domain: String,
    pub port: u16,
    pub root_secret: DerivableSecret,
    pub fm_db_path: PathBuf,
    pub pg_db: String,
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
            root_secret: Self::create_root_secret(&env::var("SECRET_KEY")?),
            fm_db_path: env::var("FM_DB_PATH")?.parse().expect("Invalid FM_DB_PATH"),
            pg_db: env::var("DATABASE_URL")?,
        };

        info!("Loaded config");
        Ok(config)
    }

    fn create_root_secret(secret: &str) -> DerivableSecret {
        let secret_bytes: [u8; 64] =
            FromHex::from_hex(secret).expect("Invalid SECRET_KEY hex string");
        PlainRootSecretStrategy::to_root_secret(&secret_bytes)
    }
}
