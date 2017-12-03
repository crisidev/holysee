extern crate regex;

use chan::Sender;
use message::Message;

pub trait Command {
    fn execute(&mut self, &mut Message, &Sender<Message>, &Sender<Message>);
    fn get_usage(&self) -> String;
    fn get_name(&self) -> String;
    fn matches_message_text(&self, message: &Message) -> bool;
    fn stop_processing(&self) -> bool;
}

pub struct CommandDispatcher<'a> {
    commands: Vec<&'a mut Command>,
    enabled_commands: &'a[String],
}

impl<'a> CommandDispatcher<'a> {
    pub fn new(enabled_commands: &'a[String]) -> CommandDispatcher<'a> {
        CommandDispatcher { commands: vec![], enabled_commands}
    }

    pub fn is_command_enabled(&self, command: &str) -> bool {
        self.enabled_commands.into_iter().any(|x| x == command)
    }

    pub fn register(&mut self, cmd: &'a mut Command) {
        if self.is_command_enabled(&cmd.get_name()) {
            info!("Registering new command {}", cmd.get_name());
            self.commands.push(cmd);
        } else {
            warn!(
                "Command {} is disabled in settings, skipping registration",
                cmd.get_name()
            );
        }
    }

    pub fn execute(
        &mut self,
        msg: &mut Message,
        irc_sender: &Sender<Message>,
        tg_sender: &Sender<Message>,
    ) {
        for command in self.commands.as_mut_slice() {
            if command.matches_message_text(msg) {
                info!("Executing command {}", command.get_name());
                command.execute(msg, irc_sender, tg_sender);
                if command.stop_processing() {
                    debug!("Command {} stop processing", command.get_name());
                    break;
                }
            }
        }
    }
}
