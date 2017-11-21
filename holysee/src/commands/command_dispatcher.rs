extern crate regex;

use chan::Sender;

use settings;
use message::Message;

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

    pub fn is_command_enabled(&self, command: &str) -> bool {
        self.enabled_commands.into_iter().any(|x| x == command)
    }

    pub fn set_command(&mut self, cmd: Box<Command + 'a>) {
        self.command = cmd;
    }

    pub fn execute(
        &mut self,
        msg: &Message,
        irc_sender: &Sender<Message>,
        tg_sender: &Sender<Message>,
    ) {
        self.command.execute(msg, irc_sender, tg_sender);
    }
}
