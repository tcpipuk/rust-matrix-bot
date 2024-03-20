mod bot;
mod config;
mod matrix;

use config::Config;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let config_path = PathBuf::from("./config.toml");
    
    match Config::from_file(config_path).await {
        Ok(config) => {
            let mut bot = bot::Bot::new(&config).await.expect("Failed to create bot");
            bot.run().await.expect("Bot run encountered an issue");
        }
        Err(e) => eprintln!("Failed to load config: {}", e),
    }
}
