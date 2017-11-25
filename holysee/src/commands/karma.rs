extern crate regex;
extern crate serde_json;

use chan::Sender;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::error::Error;

use self::regex::{Regex, Captures};

use settings;
use message::{Message, TransportType, DestinationType};
use commands::command_dispatcher::Command;

#[derive(Debug)]
pub struct KarmaCommand<'a> {
    enabled: bool,
    karma: HashMap<String, i64>,
    command_prefix: &'a String,
    data_dir: &'a String,
}

impl<'a> KarmaCommand<'a> {
    pub fn new(
        command_prefix: &'a String,
        settings: &'a settings::Commands,
        enabled: bool,
    ) -> KarmaCommand<'a> {
        KarmaCommand {
            karma: match KarmaCommand::read_database(&settings.data_dir, "karma") {
                Ok(v) => v,
                Err(b) => {
                    error!("Error reading database: {}", b);
                    HashMap::new()
                }
            },
            enabled,
            command_prefix: command_prefix,
            data_dir: &settings.data_dir,
        }
    }

    fn read_database(data_dir: &str, name: &str) -> Result<HashMap<String, i64>, Box<Error>> {
        let filename = format!("{}/{}.json", data_dir, name);
        let filename_clone = filename.clone();
        let file = OpenOptions::new().read(true).open(filename)?;
        serde_json::from_reader(file).or_else(move |e| {
            Err(From::from(
                format!("Cannot deserialize file {}: {}", filename_clone, e),
            ))
        })
    }

    fn write_database(&self) {
        match OpenOptions::new().write(true).truncate(true).open(format!(
            "{}/{}.json",
            self.data_dir,
            &self.get_name()
        )) {
            Ok(file) => {
                if let Err(e) = serde_json::to_writer(file, &self.karma) {
                    error!("Cannot serialize file: {}", e)
                };
            }
            Err(e) => error!("Cannot open file: {}", e),
        };
    }

    fn get(&self, key: &str) -> String {
        match self.karma.get(key) {
            Some(v) => format!("karma for \"{}\": {}", key, v),
            None => format!("no karma for \"{}\"", key),
        }
    }

    fn update(&mut self, cap: &Captures, value: i64) -> String {
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
        info!("Executing KarmaCommand");
        let re_get = Regex::new(
            format!(r"^(?:{})(?:karma|riguardo)\s+(.+)$", self.command_prefix).as_ref(),
        ).unwrap();
        let re_increase = Regex::new(r"^[vV]iva\s+(.*)$|^[hH]urrah\s+(.*)$|^(\w+)\+\+$").unwrap();
        let re_decrease = Regex::new(r"^[aA]bbasso\s+(.*)$|^[fF]uck\s+(.*)$|^(\w+)\-\-$").unwrap();

        let mut karma_irc = String::new();

        // COMMAND HANDLING
        for cap in re_get.captures_iter(&msg.text) {
            debug!("Karma get captures {:#?}", cap);
            karma_irc = self.get(&cap[1]);
        }
        for cap in re_increase.captures_iter(&msg.text) {
            debug!("Karma increase captures {:#?}", cap);
            karma_irc = self.update(&cap, 1);
        }
        for cap in re_decrease.captures_iter(&msg.text) {
            debug!("Karma decrease captures {:#?}", cap);
            karma_irc = self.update(&cap, -1);
        }

        // SEND MESSAGES
        let karma_telegram = karma_irc.clone();
        let destination_irc: DestinationType = DestinationType::klone(&msg.to);
        let destination_telegram: DestinationType = DestinationType::klone(&msg.to);
        match msg.from_transport {
            TransportType::IRC => {
                to_irc.send(Message::new(
                    TransportType::Telegram,
                    karma_irc,
                    String::from("KarmaCommand"),
                    destination_irc,
                    true,
                ));
            }
            TransportType::Telegram => {
                to_telegram.send(Message::new(
                    TransportType::IRC,
                    karma_telegram,
                    String::from("KarmaCommand"),
                    destination_telegram,
                    true,
                ));
            }
        }
    }

    fn get_usage(&self) -> String {
        return String::from(
            "\
The karma command maintains a list of strings with their karma. Use
    !karma <string> or !riguardo <string>
to see the current karma,
    viva <string> or <string>++ or hurrah <string>
to increment it,
    abbasso <string> or <string>-- or fuck <string>
to decrement it.",
        );
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_name(&self) -> String {
        String::from("karma")
    }

    fn matches_message_text(&self, message: &Message) -> bool {
        let re = Regex::new(
            format!(
                r"(^{}karma\s+(.*)$|^{}riguardo\s+(.*)|^[vV]iva\s+(.*)$|^[hH]urrah\s+(.*)$|^(\w+)\+\+$|^[aA]bbasso\s+(.*)$|^[fF]uck\s+(.*)$|^(\w+)\-\-$)",
                self.command_prefix,
                self.command_prefix
            ).as_ref(),
        ).unwrap();
        re.is_match(&message.text)
    }
}
