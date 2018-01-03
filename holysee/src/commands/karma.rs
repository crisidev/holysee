extern crate regex;
extern crate serde_json;

use chan::Sender;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::error::Error;

use self::regex::{Regex, Captures};

use message::{Message, TransportType, DestinationType};
use commands::command_dispatcher::Command;

#[derive(Debug)]
pub struct KarmaCommand<'a> {
    karma: HashMap<String, i64>,
    command_prefix: &'a str,
    data_dir: &'a str,
}

impl<'a> KarmaCommand<'a> {
    pub fn new(command_prefix: &'a str, data_dir: &'a str) -> KarmaCommand<'a> {
        KarmaCommand {
            karma: match KarmaCommand::read_database(data_dir, "karma") {
                Ok(v) => v,
                Err(b) => {
                    error!("Error reading database: {}", b);
                    HashMap::new()
                }
            },
            command_prefix: command_prefix,
            data_dir: data_dir,
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

    fn write_database(&self) -> bool {
        let filename = format!("{}/{}.json", self.data_dir, &self.get_name());
        let filename_clone = filename.clone();
        match OpenOptions::new().write(true).truncate(true).open(filename) {
            Ok(file) => {
                if let Err(e) = serde_json::to_writer(file, &self.karma) {
                    error!("Cannot serialize file {}: {}", filename_clone, e);
                    return false;
                };
            }
            Err(e) => {
                error!("Cannot open file {}: {}", filename_clone, e);
                return false;
            }
        };
        true
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

    fn handle(&mut self, text: &str) -> String {
        let re_get = Regex::new(
            format!(r"^(?:{})(?:karma|riguardo)\s+(.+)$", self.command_prefix).as_ref(),
        ).unwrap();
        let re_increase = Regex::new(r"^[vV]iva\s+(.*)$|^[hH]urrah\s+(.*)$|^(\w+)\+\+$").unwrap();
        let re_decrease = Regex::new(r"^[aA]bbasso\s+(.*)$|^[fF]uck\s+(.*)$|^(\w+)\-\-$").unwrap();

        let mut result = String::new();

        // COMMAND HANDLING
        for cap in re_get.captures_iter(text) {
            debug!("Karma get captures {:#?}", cap);
            result = self.get(&cap[1]);
        }
        for cap in re_increase.captures_iter(text) {
            debug!("Karma increase captures {:#?}", cap);
            result = self.update(&cap, 1);
        }
        for cap in re_decrease.captures_iter(text) {
            debug!("Karma decrease captures {:#?}", cap);
            result = self.update(&cap, -1);
        }
        result
    }
}

impl<'a> Command for KarmaCommand<'a> {
    fn execute(
        &mut self,
        msg: &mut Message,
        to_irc: &Sender<Message>,
        to_telegram: &Sender<Message>,
    ) {
        let karma_irc = self.handle(&msg.text);
        let karma_telegram = karma_irc.clone();

        let destination = match msg.to {
            DestinationType::Channel(ref c) => DestinationType::Channel(c.clone()),
            DestinationType::User(_) => DestinationType::User(msg.from.clone()),
            DestinationType::Unknown => panic!("Serious bug in karma command handler"),
        };
        let destination_irc: DestinationType = DestinationType::klone(&destination);
        let destination_telegram: DestinationType = DestinationType::klone(&destination);

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
        String::from(
            "\
The karma command maintains a list of strings with their karma. Use
    !karma <string> or !riguardo <string>
to see the current karma,
    viva <string> or <string>++ or hurrah <string>
to increment it,
    abbasso <string> or <string>-- or fuck <string>
to decrement it.",
        )
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

    fn stop_processing(&self, _: &Message) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use self::tempdir::TempDir;

    use super::{Command, KarmaCommand, Message, TransportType, DestinationType};

    #[test]
    fn test_read_database() {
        assert!(KarmaCommand::read_database("adir", "karma.json").is_err());
        let data_dir = TempDir::new("holysee_karma").unwrap();
        assert!(
            KarmaCommand::read_database(data_dir.path().to_str().unwrap(), "karma.json").is_err()
        );
    }

    #[test]
    fn test_write_database() {
        // TODO: handle also successful case which now returns
        // Cannot open file /var/folders/xj/8kykppps3b9d79m8g40nbyz9p52bt_/T/holysee_quote.C6rvI2j6eqIa/quote.json: No such file or directory (os error 2)
        // Cannot open file /var/folders/xj/8kykppps3b9d79m8g40nbyz9p52bt_/T/holysee_quote.C6rvI2j6eqIa/quote.json: No such file or directory (os error 2)
        let prefix = String::from("!");
        let karma = KarmaCommand::new(&prefix, "adir");
        assert!(!karma.write_database());
    }

    #[test]
    fn test_matches_message_text() {
        let prefix = String::from("!");
        let data_dir = String::from("adir");
        let karma = KarmaCommand::new(&prefix, &data_dir);
        let mut msg = Message {
            from_transport: TransportType::IRC,
            text: String::from("!karma"),
            from: String::from("auser"),
            to: DestinationType::User(String::from("auser")),
            is_from_command: false,
        };

        let success = [
            "!karma something",
            "!riguardo something",
            "viva something",
            "hurrah something",
            "something++",
            "abbasso something",
            "fuck something",
            "something--",
            "Abbasso something",
        ];
        for text in success.iter() {
            msg.text = String::from(*text);
            assert!(karma.matches_message_text(&msg));
        }

        let failures = [
            "!karma",
            "!riguardo",
            "karma",
            "karma ",
            "hurra ",
            "fck ",
            "viv something",
            "something ++",
            "something + +",
            "abbaso something",
            "something-",
            "something- -",
            "Vva somethong",
            "Abasso something",
        ];
        for text in failures.iter() {
            msg.text = String::from(*text);
            assert!(!karma.matches_message_text(&msg));
        }
    }

    #[test]
    fn test_handle() {
        let prefix = String::from("!");
        let data_dir = TempDir::new("holysee_karma").unwrap();
        let mut karma = KarmaCommand::new(&prefix, data_dir.path().to_str().unwrap());

        let cases = [
            ["!karma something", "no karma for \"something\""],
            ["something++", "karma for \"something\": 1"],
            ["viva something", "karma for \"something\": 2"],
            ["something--", "karma for \"something\": 1"],
            ["abbasso something", "karma for \"something\": 0"],
            ["!karma something", "karma for \"something\": 0"],
        ];
        for case in cases.iter() {
            assert!(karma.handle(case[0]) == *case[1]);
        }
    }
}
