use std::env;
use std::path::PathBuf;

use multimint::fedimint_client::derivable_secret::DerivableSecret;
use multimint::fedimint_client::secret::{PlainRootSecretStrategy, RootSecretStrategy};
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
        let secret_vec: Vec<u8> = hex::decode(secret).expect("Invalid SECRET_KEY hex string");
        let secret_arr: [u8; 64] = secret_vec.try_into().expect("Invalid SECRET_KEY length");
        PlainRootSecretStrategy::to_root_secret(&secret_arr)
    }
}
