extern crate regex;
extern crate serde_json;
extern crate chrono;

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::error::Error;
use chan::Sender;

use self::regex::Regex;
use self::chrono::{Local, NaiveDateTime};

use message::{Message, TransportType, DestinationType};
use commands::command_dispatcher::Command;

#[derive(Debug)]
pub struct LastSeenCommand<'a> {
    last_seen: HashMap<String, i64>,
    command_prefix: &'a str,
    data_dir: &'a str,
}

impl<'a> LastSeenCommand<'a> {
    pub fn new(command_prefix: &'a str, data_dir: &'a str) -> LastSeenCommand<'a> {
        LastSeenCommand {
            last_seen: match LastSeenCommand::read_database(data_dir, "last_seen") {
                Ok(v) => v,
                Err(b) => {
                    error!("Error reading database: {}", b);
                    HashMap::new()
                }
            },
            command_prefix,
            data_dir: data_dir,
        }
    }

    fn read_database(data_dir: &str, name: &str) -> Result<HashMap<String, i64>, Box<Error>> {
        let filename = format!("{}/{}.json", data_dir, name);
        let filename_clone = filename.clone();
        let file = OpenOptions::new().read(true).open(filename)?;
        serde_json::from_reader(file).or_else(|e| {
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
                if let Err(e) = serde_json::to_writer(file, &self.last_seen) {
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

    fn handle(&mut self, text: &str, from: &str) -> String {
        let re_get = Regex::new(
            format!(r"^(?:{})seen\s+(.*)$", &self.command_prefix).as_ref(),
        ).unwrap();

        let mut result = String::new(); 
        // COMMAND HANDLING
        self.see(from);
        for cap in re_get.captures_iter(text) {
            debug!("Last seen captures {:#?}", cap);
            result = self.get(&cap[1]);
        }
        result
    }
}

impl<'a> Command for LastSeenCommand<'a> {
    fn execute(
        &mut self,
        msg: &mut Message,
        to_irc: &Sender<Message>,
        to_telegram: &Sender<Message>,
    ) {
        let last_seen_irc = self.handle(&msg.text, &msg.from);
        if last_seen_irc != "" {
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

    fn stop_processing(&self, msg: &Message) -> bool {
        // TODO abstract the "seen" string to another so that we do not duplicate the changes
        let pattern = format!("{}seen", &self.command_prefix);
        msg.text.contains(&pattern)
    }
}

#[cfg(test)]
mod tests {
    extern crate tempdir;

    use self::tempdir::TempDir;

    use super::LastSeenCommand;

    #[test]
    fn test_read_database() {
        assert!(LastSeenCommand::read_database("adir", "last_seen.json").is_err());
        let data_dir = TempDir::new("holysee_last_seen").unwrap();
        assert!(
            LastSeenCommand::read_database(data_dir.path().to_str().unwrap(), "last_seen.json").is_err()
        );
    }

    #[test]
    fn test_write_database() {
        // TODO: handle also successful case which now returns
        // Cannot open file /var/folders/xj/8kykppps3b9d79m8g40nbyz9p52bt_/T/holysee_last_seen.C6rvI2j6eqIa/last_seen.json: No such file or directory (os error 2)
        // Cannot open file /var/folders/xj/8kykppps3b9d79m8g40nbyz9p52bt_/T/holysee_last_seen.C6rvI2j6eqIa/last_seen.json: No such file or directory (os error 2)
        let prefix = String::from("!");
        let karma = LastSeenCommand::new(&prefix, "adir");
        assert!(!karma.write_database());
    }

    #[test]
    fn test_handle() {
        let prefix = String::from("!");
        let data_dir = TempDir::new("holysee_last_seen").unwrap();
        let mut seen = LastSeenCommand::new(&prefix, data_dir.path().to_str().unwrap());

        let cases = [
            ["!seen", "auser"],
            ["!Seen", "auser"],
            ["!seen", "anotheruser"],
            ["!Seen", "anotheruser"],
        ];

        for case in cases.iter() {
            println!("{}", seen.handle(case[0], case[1]));
            assert!(seen.handle(case[0], case[1]).is_empty());
        }

        // TODO: actually implement testing when a user is seen
    }
}
