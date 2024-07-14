use std::sync::Arc;

use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::log;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

use crate::config::Config;

#[derive(Clone)]
pub struct AppData {
    db: DatabaseConnection,
    vault: Arc<VaultClient>,
}

impl AppData {
    pub async fn new(config: &Config) -> Self {
        let mut opt = ConnectOptions::new(config.database_url.to_owned());
        opt.sqlx_logging(true)
            .sqlx_logging_level(log::LevelFilter::Debug);

        let db = Database::connect(opt).await.expect("Connection error");

        let vault = VaultClient::new(
            VaultClientSettingsBuilder::default()
                .address(config.vault_storage.to_string())
                .token(config.vault_token.to_string())
                .build()
                .expect("Vault client settings error"),
        )
        .expect("Vault client error");

        AppData {
            db,
            vault: Arc::new(vault),
        }
    }

    pub fn get_db_connection(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn get_vault_client(&self) -> Arc<VaultClient> {
        self.vault.clone()
    }
}
