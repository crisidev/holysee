extern crate regex;
extern crate serde_json;

use message::{Message, TransportType};
use chan::Sender;
use std::collections::HashMap;
use std::fs::File;
use std::ops::Add;

use self::regex::Regex;

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
pub struct RelayMessageCommand {
    irc_allow_receive: bool,
    telegram_allow_receive: bool,
    command_prefix: String,
}

impl RelayMessageCommand {
    pub fn new(
        irc_allow_receive: bool,
        telegram_allow_receive: bool,
        get_command_prefix: &Fn() -> String,
    ) -> RelayMessageCommand {
        RelayMessageCommand {
            irc_allow_receive,
            telegram_allow_receive,
            command_prefix: String::from(get_command_prefix()),
        }
    }
    pub fn matches_message_text(&self, message: &Message) -> bool {
        match message.from_transport {
            TransportType::IRC => {
                if self.telegram_allow_receive {
                    return true;
                }
            }
            TransportType::Telegram => {
                if self.irc_allow_receive {
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

impl Command for RelayMessageCommand {
    fn execute(
        &mut self,
        msg: &Message,
        irc_sender: &Sender<Message>,
        telegram_sender: &Sender<Message>,
    ) {
        match msg.from_transport {
            TransportType::IRC => {
                debug!("MessageAsCommand::to_telegram");
                telegram_sender.send(Message {
                    text: msg.strip_command(&self.command_prefix),
                    from: msg.from.clone(),
                    to: msg.to.clone(),
                    from_transport: TransportType::IRC,
                });
            }
            TransportType::Telegram => {
                debug!("MessageAsCommand::to_irc");
                irc_sender.send(Message {
                    text: msg.strip_command(&self.command_prefix),
                    from: msg.from.clone(),
                    to: msg.to.clone(),
                    from_transport: TransportType::Telegram,
                });
            }
        }
    }
}

#[derive(Debug)]
pub struct KarmaCommand {
    karma: HashMap<String, i64>,
    command_prefix: String,
}

impl KarmaCommand {
    pub fn new(get_command_prefix: &Fn() -> String) -> KarmaCommand {
        // load the current known karma
        // TODO: abstract file name and path
        let file = File::open("data/karma.json").unwrap();
        let karma = serde_json::from_reader(file).unwrap();
        KarmaCommand {
            karma,
            command_prefix: String::from(get_command_prefix()),
        }
    }
    pub fn matches_message_text(&self, message: &Message) -> bool {
        let re = Regex::new(
            format!(r"(^{}karma (.*)$|^viva (.*)$|^(\w+)\+\+$|^abbasso (.*)$|^(\w+)\-\-$)", self.command_prefix).as_ref()
        ).unwrap();
        re.is_match(&message.text)
    }
}

impl Command for KarmaCommand {
    fn execute(&mut self, msg: &Message, to_irc: &Sender<Message>, to_telegram: &Sender<Message>) {
        debug!("karma execute");
        let re_get = Regex::new(format!(r"^{}karma (.*)$", self.command_prefix).as_ref()).unwrap();
        let re_increment = Regex::new(r"^viva (.*)$|^(\w+)\+\+$").unwrap();
        let re_decrease = Regex::new(r"^abbasso (.*)$|^(\w+)\-\-$").unwrap();

        let mut karma_irc = String::new();
        let mut karma_telegram = String::new();

        for cap in re_get.captures_iter(&msg.text) {
            debug!("Karma request for captures {:#?}", &cap[1]);
            karma_irc = match self.karma.get(&cap[1]) {
                Some(v) => format!("karma for \"{}\": {}", &cap[1], v),
                None => format!("no karma for \"{}\"", &cap[1])
            };
            karma_telegram = karma_irc.clone();
        }
        for cap in re_increment.captures_iter(&msg.text) {
            for group in cap.iter() {
                match group {
                    Some(x) => {
                        *(self.karma.entry(String::from(x.as_str())).or_insert(0)) += 1;
                        karma_irc = match self.karma.get(x.as_str()) {
                            Some(v) => format!("updated karma for \"{}\": {}", x.as_str(), v),
                            None => format!("created karma for \"{}\"", x.as_str())
                        };
                    },
                    None => continue,
                }
            }
            karma_telegram = karma_irc.clone();
        }
        to_irc.send(Message {
            from: String::from("KarmaCommand"),
            text: String::from(karma_irc),
            from_transport: TransportType::Telegram,
            to: String::from("karma"),
        });
        to_telegram.send(Message {
            from: String::from("KarmaCommand"),
            text: String::from(karma_telegram),
            from_transport: TransportType::IRC,
            to: String::from("karma"),
        });
    }
}

pub struct CommandDispatcher<'a> {
    command: Box<Command + 'a>,
}

impl<'a> CommandDispatcher<'a> {
    pub fn new() -> CommandDispatcher<'a> {
        CommandDispatcher { command: Box::new(NullCommand::new()) }
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
