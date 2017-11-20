extern crate regex;
extern crate serde_json;
extern crate chrono;

use self::chrono::{Local, NaiveDateTime};
use message::{Message, TransportType};
use chan::Sender;
use std::collections::HashMap;
use std::fs::OpenOptions;
use settings;

use self::regex::{Regex, Captures};

pub trait Command {
    fn execute(&mut self, &Message, &Sender<Message>, &Sender<Message>);
}

#[derive(Debug)]
struct NullCommand;

impl NullCommand {
    fn new() -> NullCommand {
        NullCommand
    }
}

impl Command for NullCommand {
    fn execute(&mut self, _: &Message, _: &Sender<Message>, _: &Sender<Message>) {}
}

#[derive(Debug)]
pub struct RelayMessageCommand<'a> {
    pub name: String,
    irc_allow_receive: &'a bool,
    telegram_allow_receive: &'a bool,
    command_prefix: &'a String,
}

impl<'a> RelayMessageCommand<'a> {
    pub fn new(
        irc_allow_receive: &'a bool,
        telegram_allow_receive: &'a bool,
        command_prefix: &'a String,
    ) -> RelayMessageCommand<'a> {
        RelayMessageCommand {
            name: String::from("relay"),
            irc_allow_receive,
            telegram_allow_receive,
            command_prefix,
        }
    }
    pub fn matches_message_text(&self, message: &Message) -> bool {
        match message.from_transport {
            TransportType::IRC => {
                if *self.telegram_allow_receive || message.is_from_command {
                    return true;
                }
            }
            TransportType::Telegram => {
                if *self.irc_allow_receive || message.is_from_command {
                    return true;
                }
            }
        };
        let re = Regex::new(
            format!(r"^({})(irc|tg)\s+(.*)$", self.command_prefix).as_ref(),
        ).unwrap();
        re.is_match(&message.text)
    }
}

impl<'a> Command for RelayMessageCommand<'a> {
    fn execute(
        &mut self,
        msg: &Message,
        irc_sender: &Sender<Message>,
        telegram_sender: &Sender<Message>,
    ) {
        match msg.from_transport {
            TransportType::IRC => {
                debug!("MessageAsCommand::to_telegram");
                telegram_sender.send(Message::new(
                    TransportType::IRC,
                    msg.strip_command(&self.command_prefix),
                    msg.from.clone(),
                    msg.to.clone(),
                    msg.is_from_command,
                ));
            }
            TransportType::Telegram => {
                debug!("MessageAsCommand::to_irc");
                irc_sender.send(Message::new(
                    TransportType::Telegram,
                    msg.strip_command(&self.command_prefix),
                    msg.from.clone(),
                    msg.to.clone(),
                    msg.is_from_command,
                ));
            }
        }
    }
}

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
        let re_get = Regex::new(format!(r"^(?:{})karma\s+(.*)$", self.command_prefix).as_ref())
            .unwrap();
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


#[derive(Debug)]
pub struct LastSeenCommand<'a> {
    pub name: String,
    last_seen: HashMap<String, i64>,
    command_prefix: &'a String,
    data_dir: &'a String,
}

impl<'a> LastSeenCommand<'a> {
    pub fn new(command_prefix: &'a String, settings: &'a settings::Commands) -> LastSeenCommand<'a> {
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
                match serde_json::to_writer(file, &self.last_seen) {
                    Err(e) => error!("cannot serialize file: {}", e),
                    _ => {}
                };
            }
            Err(e) => error!("cannot open file: {}", e),
        };
    }

    fn get(&self, key: &str) -> String {
        match self.last_seen.get(key) {
            Some(v) => format!("last seen \"{}\": {}", key, NaiveDateTime::from_timestamp(*v, 0).format("%Y-%m-%d %H:%M:%S")),
            None => format!("never seen \"{}\"", key),
        }
    }

    fn see(&mut self, who: &String) {
        *(self.last_seen.entry(String::from(who.clone())).or_insert(Local::now().timestamp())) = Local::now().timestamp();
        self.write_database();
    }
}

impl<'a> Command for LastSeenCommand<'a> {
    fn execute(&mut self, msg: &Message, to_irc: &Sender<Message>, to_telegram: &Sender<Message>) {
        debug!("last_seen execute");
        let re_get = Regex::new(format!(r"^(?:{})seen\s+(.*)$", &self.command_prefix).as_ref()).unwrap();

        // COMMAND HANDLING
        self.see(&msg.from);
        for cap in re_get.captures_iter(&msg.text) {
            debug!("last_seen for captures {:#?}", cap);
            let last_seen_irc = self.get(&cap[1]);
            // SEND MESSAGES
            let last_seen_telegram = last_seen_irc.clone();
            to_irc.send(Message::new(
                TransportType::Telegram,
                String::from(last_seen_irc),
                String::from("LastSeenCommand"),
                String::from(self.name.clone()),
                true,
            ));
            to_telegram.send(Message::new(
                TransportType::IRC,
                String::from(last_seen_telegram),
                String::from("LastSeenCommand"),
                String::from(self.name.clone()),
                true,
            ));
        }
    }
}


pub struct CommandDispatcher<'a> {
    command: Box<Command + 'a>,
    enabled_commands: &'a Vec<String>,
}

impl<'a> CommandDispatcher<'a> {
    pub fn new(settings: &'a settings::Commands) -> CommandDispatcher<'a> {
        CommandDispatcher {
            command: Box::new(NullCommand::new()),
            enabled_commands: &settings.enabled,
        }
    }

    pub fn is_command_enabled(&self, command: &String) -> bool {
        self.enabled_commands
            .into_iter()
            .find(|&x| x == command)
            .is_some()
    }

    pub fn set_command(&mut self, cmd: Box<Command + 'a>) {
        debug!("set_command in CommandDispatcher");
        self.command = cmd;
    }

    pub fn execute(
        &mut self,
        msg: &Message,
        irc_sender: &Sender<Message>,
        tg_sender: &Sender<Message>,
    ) {
        debug!("execute in CommandDispatcher");
        self.command.execute(msg, irc_sender, tg_sender);
    }
}
