use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub homeserver_url: String,
    pub user_id: String,
    pub device_id: Option<String>,
    pub password: Option<String>,
    pub auth_token: Option<String>,
}

impl Config {
    pub async fn from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path).await?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub async fn save_to_file(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let toml = toml::to_string(self)?;
        fs::write(path, toml).await?;
        Ok(())
    }

    // Helper function to determine if the config needs to login and generate a new token
    pub fn needs_login(&self) -> bool {
        self.auth_token.is_none() && self.password.is_some()
    }

    // Helper function to update the device_id and auth_token in the config
    pub fn update_auth_details(&mut self, device_id: String, auth_token: String) {
        self.device_id = Some(device_id);
        self.auth_token = Some(auth_token);
    }
}
