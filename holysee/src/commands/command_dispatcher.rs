extern crate regex;

use chan::Sender;
use message::Message;

pub trait Command {
    fn execute(&mut self, &Message, &Sender<Message>, &Sender<Message>);
    fn get_usage(&self) -> String;
    fn is_enabled(&self) -> bool;
    fn get_name(&self) -> String;
    fn matches_message_text(&self, message: &Message) -> bool;
}

pub struct CommandDispatcher<'a> {
    commands: Vec<&'a mut Command>,
}

impl<'a> CommandDispatcher<'a> {
    pub fn new() -> CommandDispatcher<'a> {
        CommandDispatcher { commands: vec![] }
    }

    pub fn register(&mut self, cmd: &'a mut Command) {
        self.commands.push(cmd);
    }

    pub fn execute(
        &mut self,
        msg: &Message,
        irc_sender: &Sender<Message>,
        tg_sender: &Sender<Message>,
    ) {
        for command in self.commands.as_mut_slice() {
            if command.matches_message_text(msg) {
                debug!("execute() for {}", command.get_name());
                command.execute(msg, irc_sender, tg_sender);
            }
        }
    }
}
