extern crate regex;
extern crate serde_json;
extern crate chrono;

use self::chrono::{Local, NaiveDateTime};
use message::{Message, TransportType};
use chan::Sender;
use std::collections::HashMap;
use std::fs::OpenOptions;
use settings;

use self::regex::Regex;
use commands::command_dispatcher::Command;

#[derive(Debug)]
pub struct LastSeenCommand<'a> {
    pub name: String,
    last_seen: HashMap<String, i64>,
    command_prefix: &'a String,
    data_dir: &'a String,
}

impl<'a> LastSeenCommand<'a> {
    pub fn new(
        command_prefix: &'a String,
        settings: &'a settings::Commands,
    ) -> LastSeenCommand<'a> {
        LastSeenCommand {
            name: String::from("last_seen"),
            last_seen: LastSeenCommand::read_database(&settings.data_dir, "last_seen"),
            command_prefix,
            data_dir: &settings.data_dir,
        }
    }
    pub fn matches_message_text(&self, _: &Message) -> bool {
        true
    }

    fn read_database(data_dir: &String, name: &str) -> HashMap<String, i64> {
        // load the current known seen times
        match OpenOptions::new().read(true).open(format!(
            "{}/{}.json",
            data_dir,
            name
        )) {
            Ok(file) => {
                match serde_json::from_reader(file) {
                    Err(e) => {
                        error!("cannot deserialize file: {}", e);
                        HashMap::new()
                    }
                    Ok(k) => k,
                }
            }
            Err(e) => {
                error!("cannot open file: {}", e);
                HashMap::new()
            }
        }
    }

    fn write_database(&self) {
        match OpenOptions::new().write(true).open(format!(
            "{}/{}.json",
            self.data_dir,
            &self.name
        )) {
            Ok(file) => {
                if let Err(e) = serde_json::to_writer(file, &self.last_seen) {
                    error!("cannot serialize file: {}", e)
                };
            }
            Err(e) => error!("cannot open file: {}", e),
        };
    }

    fn get(&self, key: &str) -> String {
        match self.last_seen.get(key) {
            Some(v) => {
                format!(
                    "last seen \"{}\": {}",
                    key,
                    NaiveDateTime::from_timestamp(*v, 0).format("%Y-%m-%d %H:%M:%S")
                )
            }
            None => format!("never seen \"{}\"", key),
        }
    }

    fn see(&mut self, who: &str) {
        *(self.last_seen.entry(who.to_owned()).or_insert(
            Local::now().timestamp(),
        )) = Local::now().timestamp();
        self.write_database();
    }
}

impl<'a> Command for LastSeenCommand<'a> {
    fn execute(&mut self, msg: &Message, to_irc: &Sender<Message>, to_telegram: &Sender<Message>) {
        debug!("last_seen execute");
        let re_get = Regex::new(
            format!(r"^(?:{})seen\s+(.*)$", &self.command_prefix).as_ref(),
        ).unwrap();

        // COMMAND HANDLING
        self.see(&msg.from);
        for cap in re_get.captures_iter(&msg.text) {
            debug!("last_seen for captures {:#?}", cap);
            let last_seen_irc = self.get(&cap[1]);
            // SEND MESSAGES
            let last_seen_telegram = last_seen_irc.clone();
            to_irc.send(Message::new(
                TransportType::Telegram,
                last_seen_irc,
                String::from("LastSeenCommand"),
                self.name.to_owned(),
                true,
            ));
            to_telegram.send(Message::new(
                TransportType::IRC,
                last_seen_telegram,
                String::from("LastSeenCommand"),
                self.name.to_owned(),
                true,
            ));
        }
    }
}
