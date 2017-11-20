extern crate regex;
extern crate serde_json;
extern crate chrono;

use message::{Message, TransportType};
use chan::Sender;
use std::collections::HashMap;
use std::fs::OpenOptions;
use settings;
use commands::command_dispatcher::Command;

use self::regex::{Regex, Captures};

#[derive(Debug)]
pub struct KarmaCommand<'a> {
    pub name: String,
    karma: HashMap<String, i64>,
    command_prefix: &'a String,
    data_dir: &'a String,
}

impl<'a> KarmaCommand<'a> {
    pub fn new(command_prefix: &'a String, settings: &'a settings::Commands) -> KarmaCommand<'a> {
        KarmaCommand {
            name: String::from("karma"),
            karma: KarmaCommand::read_database(&settings.data_dir, "karma"),
            command_prefix: command_prefix,
            data_dir: &settings.data_dir,
        }
    }
    pub fn matches_message_text(&self, message: &Message) -> bool {
        let re = Regex::new(
            format!(
                r"(^{}karma\s+(.*)$|^[vV]iva\s+(.*)$|^(\w+)\+\+$|^[aA]bbasso\s+(.*)$|^(\w+)\-\-$)",
                self.command_prefix
            ).as_ref(),
        ).unwrap();
        re.is_match(&message.text)
    }

    fn read_database(data_dir: &String, name: &str) -> HashMap<String, i64> {
        // load the current known karma
        // TODO: abstract file name and path
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
                match serde_json::to_writer(file, &self.karma) {
                    Err(e) => error!("cannot serialize file: {}", e),
                    _ => {}
                };
            }
            Err(e) => error!("cannot open file: {}", e),
        };
    }

    fn get(&self, key: &str) -> String {
        match self.karma.get(key) {
            Some(v) => format!("karma for \"{}\": {}", key, v),
            None => format!("no karma for \"{}\"", key),
        }
    }

    fn edit(&mut self, cap: Captures, value: i64) -> String {
        let mut karma_irc = String::new();
        for group in cap.iter().skip(1) {
            match group {
                Some(x) => {
                    *(self.karma.entry(String::from(x.as_str())).or_insert(0)) += value;
                    karma_irc = self.get(x.as_str());
                    self.write_database();
                }
                None => continue,
            }
        }
        karma_irc
    }
}

impl<'a> Command for KarmaCommand<'a> {
    fn execute(&mut self, msg: &Message, to_irc: &Sender<Message>, to_telegram: &Sender<Message>) {
        debug!("karma execute");
        let re_get = Regex::new(
            format!(r"^(?:{})karma\s+(.*)$", self.command_prefix).as_ref(),
        ).unwrap();
        let re_increase = Regex::new(r"^[vV]iva\s+(.*)$|^(\w+)\+\+$").unwrap();
        let re_decrease = Regex::new(r"^[aA]bbasso\s+(.*)$|^(\w+)\-\-$").unwrap();

        let mut karma_irc = String::new();

        // COMMAND HANDLING
        for cap in re_get.captures_iter(&msg.text) {
            debug!("karma get for captures {:#?}", cap);
            karma_irc = self.get(&cap[1]);
        }
        for cap in re_increase.captures_iter(&msg.text) {
            debug!("karma increase for captures {:#?}", cap);
            karma_irc = self.edit(cap, 1);
        }
        for cap in re_decrease.captures_iter(&msg.text) {
            debug!("karma decrease for captures {:#?}", cap);
            karma_irc = self.edit(cap, -1);
        }

        // SEND MESSAGES
        let karma_telegram = karma_irc.clone();
        to_irc.send(Message::new(
            TransportType::Telegram,
            String::from(karma_irc),
            String::from("KarmaCommand"),
            String::from("karma"),
            true,
        ));
        to_telegram.send(Message::new(
            TransportType::IRC,
            String::from(karma_telegram),
            String::from("KarmaCommand"),
            String::from("karma"),
            true,
        ));
    }
}