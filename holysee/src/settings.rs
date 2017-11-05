use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct Irc {
    pub nickname: String,
    pub username: String,
    pub real_name: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub channel: String,
    pub ssl: bool,
    pub ssl_verify: bool,
    pub allow_receive: bool,
}

#[derive(Debug, Deserialize)]
pub struct Telegram {
    pub token: String,
    pub chat_id: i64,
    pub allow_receive: bool,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub command_prefix: String,
    pub irc: Irc,
    pub telegram: Telegram,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        debug!("Creating new configuration");
        let mut s = Config::new();
        s.merge(File::with_name("config/local"));
        s.deserialize()
    }
}
