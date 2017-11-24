extern crate regex;

use self::regex::Regex;

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
    pub fn klone(other: &DestinationType) -> DestinationType{
        match other {
            &DestinationType::Channel(ref s) => {
                DestinationType::Channel(String::from(s.clone()))
            },
            &DestinationType::User(ref u) => {
                DestinationType::Channel(String::from(u.clone()))
            },
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
}
