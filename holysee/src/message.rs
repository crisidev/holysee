extern crate regex;

use self::regex::Regex;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub enum TransportType {
    IRC,
    Telegram,
}

impl TransportType {
    pub fn create_capture_regex(&self) -> regex::Regex {
        match *self {
            TransportType::IRC => {
                Regex::new(r"tg (.*)").unwrap()
            },
            TransportType::Telegram => {
                Regex::new(r"irc (.*)").unwrap()
            },
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub from_transport: TransportType,
    pub text: String,
    pub from: String,
    pub to: String,
}

impl Message {
    pub fn format(&self) -> String {
        format!("{}: {}", self.from, self.text)
    }

    pub fn is_command(&self, command_prefix: &str) -> bool {
        self.text.starts_with(command_prefix)
    }

    pub fn handle_command(&self, irc_sender: Sender<Message>, tg_sender: Sender<Message>) {
        match (*self).from_transport {
            TransportType::Telegram => {
                for cap in TransportType::Telegram.create_capture_regex().captures_iter(self.text.as_ref()) {
                    let new_message = Message{
                        // entire capture group in [0]
                        text: String::from(&cap[1]),
                        from_transport: TransportType::Telegram,
                        to: (*self).to.clone(),
                        from: (*self).from.clone(),
                    };
                    irc_sender.send(new_message).unwrap();
                }
            },
            TransportType::IRC => {
                for cap in TransportType::IRC.create_capture_regex().captures_iter(self.text.as_ref()) {
                    let new_message = Message{
                        // entire capture group in [0]
                        text: String::from(&cap[1]),
                        from_transport: TransportType::IRC,
                        to: (*self).to.clone(),
                        from: (*self).from.clone(),
                    };
                    tg_sender.send( new_message).unwrap();
                }
            },
        };
    }
}