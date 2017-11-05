extern crate regex;


use message::{Message, TransportType};
use std::sync::mpsc::Sender;

use self::regex::Regex;

pub trait Command {
    fn execute(&self, &Message, Sender<Message>, Sender<Message>);
}

#[derive(Debug)]
struct NullCommand;

impl NullCommand {
    fn new() -> NullCommand {
        NullCommand
    }
}

impl Command for NullCommand {
    fn execute(&self, _: &Message, _: Sender<Message>, _: Sender<Message>) {}
}

#[derive(Debug, Copy, Clone)]
pub struct MessageAsCommand;

impl MessageAsCommand {
    pub fn new() -> MessageAsCommand {
        MessageAsCommand
    }

    pub fn to_irc(&self, msg: &Message, irc_sender: Sender<Message>) {
        debug!("MessageAsCommand::to_irc");
        irc_sender.send(Message {
            text: msg.text.clone(),
            from: msg.from.clone(),
            to: msg.to.clone(),
            from_transport: TransportType::Telegram,
        }).unwrap();
    }
    pub fn to_telegram(&self, msg: &Message, tg_sender: Sender<Message>) {
        debug!("MessageAsCommand::to_irc");
        tg_sender.send(Message {
            text: msg.text.clone(),
            from: msg.from.clone(),
            to: msg.to.clone(),
            from_transport: TransportType::IRC,
        }).unwrap();
    }
}

#[derive(Debug)]
pub struct SendToIrcCommand {
    command: MessageAsCommand,
}

impl SendToIrcCommand {
    pub fn new(command: MessageAsCommand) -> SendToIrcCommand {
        SendToIrcCommand { command }
    }
    pub fn matches_message_text(text: &str) -> Option<String> {
        for cap in Regex::new(r"irc (.*)").unwrap().captures_iter(text) {
            debug!("SendToIrcCommand captures {:#?}", &cap[1]);
            return Some(String::from(&cap[1]));
        }
        return None
    }
}

impl Command for SendToIrcCommand {
    fn execute(&self, msg: &Message, irc_sender: Sender<Message>, _: Sender<Message>) {
        self.command.to_irc(msg, irc_sender);
    }
}

#[derive(Debug)]
pub struct SendToTelegramCommand {
    command: MessageAsCommand,
}

impl SendToTelegramCommand {
    pub fn new(command: MessageAsCommand) -> SendToTelegramCommand {
        SendToTelegramCommand { command }
    }
    pub fn matches_message_text(text: &str) -> Option<String> {
        for cap in Regex::new(r"irc (.*)").unwrap().captures_iter(text) {
            debug!("SendToTelegramCommand captures {:#?}", &cap[1]);
            return Some(String::from(&cap[1]));
        }
        return None
    }
}

impl Command for SendToTelegramCommand {
    fn execute(&self, msg: &Message, _: Sender<Message>, tg_sender: Sender<Message>) {
        debug!("Command::execute with {:#?}", msg);
        self.command.to_telegram(msg, tg_sender);
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
    pub fn execute(&self, msg: &Message, irc_sender: Sender<Message>, tg_sender: Sender<Message>) {
        debug!("execute in CommandDispatcher");
        self.command.execute(msg, irc_sender, tg_sender);
    }
}

