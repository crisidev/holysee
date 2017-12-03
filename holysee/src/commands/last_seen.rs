extern crate regex;
extern crate serde_json;
extern crate chrono;

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::error::Error;
use chan::Sender;

use self::regex::Regex;
use self::chrono::{Local, NaiveDateTime};

use settings;
use message::{Message, TransportType, DestinationType};
use commands::command_dispatcher::Command;

#[derive(Debug)]
pub struct LastSeenCommand<'a> {
    last_seen: HashMap<String, i64>,
    command_prefix: &'a String,
    data_dir: &'a String,
    enabled: bool,
}

impl<'a> LastSeenCommand<'a> {
    pub fn new(
        command_prefix: &'a String,
        settings: &'a settings::Commands,
        enabled: bool,
    ) -> LastSeenCommand<'a> {
        LastSeenCommand {
            last_seen: match LastSeenCommand::read_database(&settings.data_dir, "last_seen") {
                Ok(v) => v,
                Err(b) => {
                    error!("Error reading database: {}", b);
                    HashMap::new()
                }
            },
            command_prefix,
            data_dir: &settings.data_dir,
            enabled,
        }
    }

    fn read_database(data_dir: &String, name: &str) -> Result<HashMap<String, i64>, Box<Error>> {
        let filename = format!("{}/{}.json", data_dir, name);
        let filename_clone = filename.clone();
        let file = OpenOptions::new().read(true).open(filename)?;
        serde_json::from_reader(file).or_else(|e| {
            Err(From::from(
                format!("Cannot deserialize file {}: {}", filename_clone, e),
            ))
        })
    }

    fn write_database(&self) {
        let filename = format!("{}/{}.json", self.data_dir, &self.get_name());
        let filename_clone = filename.clone();
        match OpenOptions::new().write(true).truncate(true).open(filename) {
            Ok(file) => {
                if let Err(e) = serde_json::to_writer(file, &self.last_seen) {
                    error!("Cannot serialize file {}: {}", filename_clone, e)
                };
            }
            Err(e) => error!("Cannot open file {}: {}", filename_clone, e),
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
    fn execute(
        &mut self,
        msg: &mut Message,
        to_irc: &Sender<Message>,
        to_telegram: &Sender<Message>,
    ) {
        let re_get = Regex::new(
            format!(r"^(?:{})seen\s+(.*)$", &self.command_prefix).as_ref(),
        ).unwrap();

        // COMMAND HANDLING
        self.see(&msg.from);
        for cap in re_get.captures_iter(&msg.text) {
            debug!("Last seen captures {:#?}", cap);
            let last_seen_irc = self.get(&cap[1]);
            let last_seen_telegram = last_seen_irc.clone();
            let destination = match msg.to {
                DestinationType::Channel(ref c) => DestinationType::Channel(c.clone()),
                DestinationType::User(_) => DestinationType::User(msg.from.clone()),
                DestinationType::Unknown => panic!("Serious bug in last_seen command handler"),
            };
            let destination_irc: DestinationType = DestinationType::klone(&destination);
            let destination_telegram: DestinationType = DestinationType::klone(&destination);
            // SEND MESSAGES
            match msg.from_transport {
                TransportType::IRC => {
                    to_irc.send(Message::new(
                        TransportType::Telegram,
                        last_seen_irc,
                        String::from("LastSeenCommand"),
                        destination_irc,
                        true,
                    ));
                }
                TransportType::Telegram => {
                    to_telegram.send(Message::new(
                        TransportType::IRC,
                        last_seen_telegram,
                        String::from("LastSeenCommand"),
                        destination_telegram,
                        true,
                    ));
                }
            }
        }
    }

    fn get_usage(&self) -> String {
        String::from(
            "\
The last_seen command keeps track of the last time a user sent a message to the channel.\
This can be accessed via the\
    !seen <nick>\
command. Note that all timestamps are relative to the server's timezone, usually UTC.",
        )
    }

    fn get_name(&self) -> String {
        String::from("last_seen")
    }

    fn matches_message_text(&self, _: &Message) -> bool {
        true
    }

    fn stop_processing(&self) -> bool {
        false
    }
}
