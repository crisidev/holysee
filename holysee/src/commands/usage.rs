extern crate regex;

use chan::Sender;
use std::collections::HashMap;

use self::regex::Regex;

use message::{Message, TransportType,DestinationType};
use commands::command_dispatcher::Command;

#[derive(Debug)]
pub struct UsageCommand<'a> {
    pub name: String,
    command_prefix: &'a String,
    commands: &'a HashMap<String, String>,
}

impl<'a> UsageCommand<'a> {
    pub fn new(
        command_prefix: &'a String,
        commands: &'a mut HashMap<String, String>,
    ) -> UsageCommand<'a> {
        UsageCommand {
            name: String::from("usage"),
            command_prefix,
            commands,
        }
    }

    pub fn matches_message_text(&self, message: &Message) -> bool {
        let re = Regex::new(
            // the shame cannot be forgotten
            format!(r"^(?:{})(?:[uU]sage)\s+(.*)$", self.command_prefix).as_ref(),
        ).unwrap();
        re.is_match(&message.text)
    }
}

impl<'a> Command for UsageCommand<'a> {
    fn execute(&mut self, msg: &Message, to_irc: &Sender<Message>, to_telegram: &Sender<Message>) {
        info!("Executing UsageCommand");
        let re_get = Regex::new(
            format!(r"^(?:{})usage\s+(.*)$", &self.command_prefix).as_ref(),
        ).unwrap();

        // COMMAND HANDLING
        for cap in re_get.captures_iter(&msg.text) {
            debug!("Usage captures {:#?}", cap);
            let command_name = &cap[1];
            let mut usage_string = String::new();
            for (cmd_name, cmd_usage) in self.commands.iter() {
                if *cmd_name == command_name {
                    usage_string = String::from(cmd_usage.clone());
                } else {
                    usage_string = String::from(format!("Command {} not found", command_name));
                }
            }

            let usage_string_irc = usage_string.clone();
            let usage_string_telegra = usage_string.clone();
            let destination_irc: DestinationType = DestinationType::klone(&msg.to);
            let destination_telegram: DestinationType = DestinationType::klone(&msg.to);
            // SEND MESSAGES
            to_irc.send(Message::new(
                TransportType::Telegram,
                usage_string_irc,
                String::from("UsageCommand"),
                destination_irc,
                true,
            ));
            to_telegram.send(Message::new(
                TransportType::IRC,
                usage_string_telegra,
                String::from("UsageCommand"),
                destination_telegram,
                true,
            ));
        }
    }

    fn get_usage(&self) -> String {
        return String::from("Run via !usage, it retuns this help")
    }
}
