extern crate regex;

use self::regex::Regex;
use settings::NickEntry;

#[derive(Debug)]
pub enum TransportType {
    IRC,
    Telegram,
}

#[derive(Debug, Clone)]
pub enum DestinationType {
    Channel(String),
    User(String),
    Unknown,
}

impl DestinationType {
    pub fn klone(other: &DestinationType) -> DestinationType {
        match other {
            &DestinationType::Channel(ref s) => DestinationType::Channel(String::from(s.clone())),
            &DestinationType::User(ref u) => DestinationType::User(String::from(u.clone())),
            &DestinationType::Unknown => DestinationType::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub from_transport: TransportType,
    pub text: String,
    pub from: String,
    pub to: DestinationType,
    pub is_from_command: bool,
}

impl Message {
    pub fn new(
        from_transport: TransportType,
        text: String,
        from: String,
        to: DestinationType,
        is_from_command: bool,
    ) -> Message {
        Message {
            from_transport,
            text,
            from,
            to,
            is_from_command,
        }
    }
    // TODO: sanitize this senseless abuse
    // TODO: handle symbol command for command name
    pub fn strip_command(&self, command_prefix: &str) -> String {
        let re = Regex::new(format!(r"({})\w+\s", command_prefix).as_ref()).unwrap();
        if self.is_from_command {
            format!("{}", re.replace_all(&self.text, ""))
        } else {
            format!("{}: {}", self.from, re.replace_all(&self.text, ""))
        }
    }

    // TODO: refactor this interface to not depend on settings::NickEntry
    pub fn convert_nicknames(&mut self, nicknames: &Vec<NickEntry>) {
        for nick_map in nicknames {
            match self.from_transport {
                TransportType::IRC => {
                    if self.text.contains(&nick_map.irc) {
                        debug!(
                            "Converting current irc from {} to telegram {}",
                            self.from,
                            nick_map.telegram
                        );
                        self.text = self.text.replace(&nick_map.irc, &nick_map.telegram);
                    }
                }
                TransportType::Telegram => {
                    if self.text.contains(&nick_map.telegram) {
                        debug!(
                            "Converting current telegram from {} to irc {}",
                            self.from,
                            nick_map.irc
                        );
                        self.text = self.text.replace(&nick_map.telegram, &nick_map.irc);
                    }
                }
            }
        }
    }
}
