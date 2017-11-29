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
pub struct Commands {
    pub data_dir: String,
    pub enabled: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct NickEntry {
    pub telegram: String,
    pub irc: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub command_prefix: String,
    pub irc: Irc,
    pub telegram: Telegram,
    pub commands: Commands,
    pub nicknames: Vec<NickEntry>,
}

impl Settings {
    pub fn new(prod: bool) -> Result<Self, ConfigError> {
        debug!("Loading configuration from config/local.toml");
        let mut s = Config::new();
        if prod {
            s.merge(File::with_name("config/local"));
        } else {
            s.merge(File::with_name("config/example"));
        }
        s.deserialize()
    }
}
