use config::{ConfigError, Config, File};


#[derive(Debug, Deserialize)]
pub struct Irc {
    pub nickname: String,
    pub username: String,
    pub real_name: String,
    pub password: String,
    pub host: String,
    pub port: usize,
    pub channels: Vec<String>,
    pub ssl: bool,
    pub ssl_verify: bool,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub irc: Irc,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        debug!("Creating new configuration");
        let mut s = Config::new();
        s.merge(File::with_name("config/default"));
        s.deserialize()
    }
}