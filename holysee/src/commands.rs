extern crate regex;
extern crate serde_json;

use message::{Message, TransportType};
use chan::Sender;
use std::collections::HashMap;
use std::fs::File;
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
            format!(r"^({})(irc|tg) (.*)$", self.command_prefix).as_ref(),
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
            karma: KarmaCommand::read_database(),
            command_prefix: command_prefix,
            data_dir: &settings.data_dir,
        }
    }
    pub fn matches_message_text(&self, message: &Message) -> bool {
        let re = Regex::new(
            format!(
                r"(^{}karma (.*)$|^viva (.*)$|^(\w+)\+\+$|^abbasso (.*)$|^(\w+)\-\-$)",
                self.command_prefix
            ).as_ref(),
        ).unwrap();
        re.is_match(&message.text)
    }

    fn read_database() -> HashMap<String, i64> {
        // load the current known karma
        // TODO: abstract file name and path
        match File::open("data/karma.json") {
            Ok(f) => match serde_json::from_reader(f) {
                Ok(k) => k,
                Err(e) => {
                    error!("cannot load from file: {}", e);
                    HashMap::new()
                },
            },
            Err(e) => {
                error!("cannot open file for reading: {}", e);
                HashMap::new()
            },
        }
    }

    fn write_database(&self) {
        match OpenOptions::new().write(true).open("data/karma.json") {
            Ok(file) => {
                match serde_json::to_writer(file, &self.karma) {
                    Err(e) => error!("cannot serialize file: {}", e),
                                _ => {},
                                };
                            },
                            Err(e) => error!("cannot open file: {}", e),
                        };
    }

    fn get(&self, key: &str) -> String {
        match self.karma.get(key) {
            Some(v) => format!("karma for \"{}\": {}", key, v),
            None => format!("no karma for \"{}\"", key),
        }
    }

    fn edit(&mut self, cap: Captures, text: &str, value: i64) -> String {
        let mut karma_irc = String::new();
        for group in cap.iter() {
            match group {
                Some(x) => {
                    if x.as_str() != text {
                        *(self.karma.entry(String::from(x.as_str())).or_insert(0)) += value;
                        karma_irc = self.get(x.as_str());
                        self.write_database();
                    }
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
        let re_get = Regex::new(format!(r"^(?:{})karma (.*)$", self.command_prefix).as_ref()).unwrap();
        let re_increase =  Regex::new(r"^viva (.*)$|^(\w+)\+\+$").unwrap();
        let re_decrease = Regex::new(r"^abbasso (.*)$|^(\w+)\-\-$").unwrap();

        let mut karma_irc = String::new();
        let mut karma_telegram = String::new();

        for cap in re_get.captures_iter(&msg.text) {
            debug!("karma get for captures {:#?}", cap);
            karma_irc = self.get(&cap[1]);
            karma_telegram = karma_irc.clone();
        }
        for cap in re_increase.captures_iter(&msg.text) {
            debug!("karma increase for captures {:#?}", cap);
            karma_irc = self.edit(cap, &msg.text, 1);
            karma_telegram = karma_irc.clone();
        }
        for cap in re_decrease.captures_iter(&msg.text) {
            debug!("karma decrease for captures {:#?}", cap);
            karma_irc = self.edit(cap, &msg.text, -1);
            karma_telegram = karma_irc.clone();
        }
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
