use envy::from_env;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: Option<String>,
    pub cors_origin_url: Option<String>,
    pub vault_storage: String,
    pub vault_token: String,
}

impl Default for Config {
    fn default() -> Self {
        from_env::<Config>().expect("Provide missing environment variables for Config")
    }
}
