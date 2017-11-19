extern crate regex;

use message::{Message, TransportType};
use chan::Sender;
use std::collections::HashMap;

use self::regex::Regex;

pub trait Command {
    fn execute(&self, &Message, &Sender<Message>, &Sender<Message>);
}

#[derive(Debug)]
struct NullCommand;

impl NullCommand {
    fn new() -> NullCommand {
        NullCommand
    }
}

impl Command for NullCommand {
    fn execute(&self, _: &Message, _: &Sender<Message>, _: &Sender<Message>) {}
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
                if self.irc_allow_receive {
                    return true;
                }
            }
            TransportType::Telegram => {
                if self.telegram_allow_receive {
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
        &self,
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
pub struct KarmaCommand<'a> {
    karma: HashMap<&'a str, &'a str>,
    command_prefix: String,
}

impl<'a> KarmaCommand<'a> {
    pub fn new(get_command_prefix: &Fn() -> String) -> KarmaCommand<'a> {
        KarmaCommand {
            karma: HashMap::new(),
            command_prefix: String::from(get_command_prefix()),
        }
    }
    pub fn matches_message_text(&self, message: &Message) -> bool {
        let re = Regex::new(
            r"(^!karma (.*)$|^viva (.*)$|^(\w+)\+\+$|^abbasso (.*)$|^(\w+)\-\-$)",
        ).unwrap();
        re.is_match(&message.text)
    }
}

impl<'a> Command for KarmaCommand<'a> {
    fn execute(&self, _: &Message, to_irc: &Sender<Message>, to_telegram: &Sender<Message>) {
        debug!("karma execute");
        to_irc.send(Message {
            from: String::from("KARMACOMMAND"),
            text: String::from("synthetic to IRC"),
            from_transport: TransportType::Telegram,
            to: String::from("fake_tg_user"),
        });
        to_telegram.send(Message {
            from: String::from("KARMACOMMAND"),
            text: String::from("synthetic to TG"),
            from_transport: TransportType::IRC,
            to: String::from("fake_irc_user"),
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
        &self,
        msg: &Message,
        irc_sender: &Sender<Message>,
        tg_sender: &Sender<Message>,
    ) {
        debug!("execute in CommandDispatcher");
        self.command.execute(msg, irc_sender, tg_sender);
    }
}
