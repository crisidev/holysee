extern crate regex;

use chan::Sender;

use settings;
use message::Message;

pub trait Command {
    fn execute(&mut self, &Message, &Sender<Message>, &Sender<Message>);
    fn get_usage(&self) -> String;
}

#[derive(Debug)]
pub struct NullCommand;

impl NullCommand {
    pub fn new() -> NullCommand {
        NullCommand
    }
}

impl Command for NullCommand {
    fn execute(&mut self, _: &Message, _: &Sender<Message>, _: &Sender<Message>) {}
    fn get_usage(&self) -> String {
        return String::from("null usage");
    }
}

pub struct CommandDispatcher<'a> {
    command: &'a Command,
    enabled_commands: &'a Vec<String>,
}

impl<'a> CommandDispatcher<'a> {
    pub fn new(
        settings: &'a settings::Commands,
        base_command: &'a Command,
    ) -> CommandDispatcher<'a> {
        CommandDispatcher {
            command: base_command,
            enabled_commands: &settings.enabled,
        }
    }

    pub fn is_command_enabled(&self, command: &str) -> bool {
        self.enabled_commands.into_iter().any(|x| x == command)
    }

    pub fn set_command(&mut self, cmd: &'a Command) {
        self.command = cmd;
    }

    pub fn execute(
        &self,
        msg: &Message,
        irc_sender: &Sender<Message>,
        tg_sender: &Sender<Message>,
    ) {
        self.command.execute(msg, irc_sender, tg_sender);
    }
}
