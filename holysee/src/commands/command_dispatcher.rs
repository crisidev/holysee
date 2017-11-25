extern crate regex;

use chan::Sender;

//use settings;
use message::Message;

pub trait Command {
    fn execute(&mut self, &Message, &Sender<Message>, &Sender<Message>);
    fn get_usage(&self) -> String;
    fn is_enabled(&self) -> bool;
    fn get_name(&self) -> String;
    fn matches_message_text(&self, message: &Message) -> bool;
}
//
//#[derive(Debug)]
//pub struct NullCommand;
//
//impl NullCommand {
//    pub fn new() -> NullCommand {
//        NullCommand
//    }
//}
//
//impl Command for NullCommand {
//    fn execute(&mut self, _: &Message, _: &Sender<Message>, _: &Sender<Message>) {}
//    fn get_usage(&self) -> String {
//        return String::from("null usage");
//    }
//    fn is_enabled(&self) -> bool {
//        false
//    }
//    fn get_name(&self) -> String {
//        String::from("null_command")
//    }
//    fn matches_message_text(&self, _: &Message) -> bool {
//        false
//    }
//}

pub struct CommandDispatcher<'a> {
    commands: Vec<&'a mut Command>,
//    enabled_commands: &'a Vec<String>,
}

impl<'a> CommandDispatcher<'a> {
    pub fn new(
//        settings: &'a settings::Commands,
    ) -> CommandDispatcher<'a> {
        CommandDispatcher {
            commands: vec!(),
//            enabled_commands: &settings.enabled,
        }
    }

//    pub fn is_command_enabled(&self, command: &str) -> bool {
//        self.enabled_commands.into_iter().any(|x| x == command)
//    }

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
