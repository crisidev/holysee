extern crate regex;

use chan::Sender;
use std::collections::HashMap;

use self::regex::Regex;

use message::{Message, TransportType, DestinationType};
use commands::command_dispatcher::Command;

#[derive(Debug)]
pub struct UsageCommand<'a> {
    command_prefix: &'a String,
    commands: &'a HashMap<String, String>,
    enabled: bool,
}

impl<'a> UsageCommand<'a> {
    pub fn new(
        command_prefix: &'a String,
        commands: &'a mut HashMap<String, String>,
        enabled: bool,
    ) -> UsageCommand<'a> {
        debug!(
            "Created usage command with usages for: {:#?}",
            commands.keys().collect::<Vec<&String>>()
        );
        UsageCommand {
            command_prefix,
            commands,
            enabled,
        }
    }
}

impl<'a> Command for UsageCommand<'a> {
    fn execute(
        &mut self,
        msg: &mut Message,
        to_irc: &Sender<Message>,
        to_telegram: &Sender<Message>,
    ) {
        let re_self = Regex::new(
            format!(r"^(?:{})(?:[uU]sage|[hH]elp)$", &self.command_prefix).as_ref(),
        ).unwrap();
        let re_get = Regex::new(
            format!(r"^(?:{})(?:[uU]sage|[hH]elp)\s+(.+)$", &self.command_prefix).as_ref(),
        ).unwrap();

        let mut usage_string = String::new();

        // COMMAND HANDLING
        for cap in re_self.captures_iter(&msg.text) {
            debug!("Usage self captures: {:#?}", cap);
            usage_string = self.get_usage();
        }

        for cap in re_get.captures_iter(&msg.text) {
            debug!("Usage argument captures: {:#?}", cap);
            let command_name = &cap[1];
            for (cmd_name, cmd_usage) in self.commands.iter() {
                if *cmd_name == command_name {
                    usage_string = cmd_usage.clone();
                    break;
                } else {
                    usage_string = format!(
                        "Command {} not found. Available ones: {:#?}",
                        command_name,
                        self.commands.keys().collect::<Vec<&String>>()
                    );
                }
            }
        }


        let usage_string_irc = usage_string.clone();
        let usage_string_telegram = usage_string.clone();
        let destination = match msg.to {
            DestinationType::Channel(ref c) => DestinationType::Channel(c.clone()),
            DestinationType::User(_) => DestinationType::User(msg.from.clone()),
            DestinationType::Unknown => panic!("Serious bug in usage command handler"),
        };
        let destination_irc: DestinationType = DestinationType::klone(&destination);
        let destination_telegram: DestinationType = DestinationType::klone(&destination);
        // SEND MESSAGES
        match msg.from_transport {
            TransportType::IRC => {
                to_irc.send(Message::new(
                    TransportType::Telegram,
                    usage_string_irc,
                    String::from("UsageCommand"),
                    destination_irc,
                    true,
                ));
            }
            TransportType::Telegram => {
                to_telegram.send(Message::new(
                    TransportType::IRC,
                    usage_string_telegram,
                    String::from("UsageCommand"),
                    destination_telegram,
                    true,
                ));
            }
        }
    }

    fn get_usage(&self) -> String {
        format!(
            "This command returns the list of available commands and their usage.\
        Available commands: {:#?}",
            self.commands.keys().collect::<Vec<&String>>()
        )
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn get_name(&self) -> String {
        String::from("usage")
    }

    fn matches_message_text(&self, message: &Message) -> bool {
        let re = Regex::new(
            // the shame cannot be forgotten
            format!(r"^(?:{})(?:[uU]sage|[hH]elp)\s*(.*)$", self.command_prefix).as_ref(),
        ).unwrap();
        re.is_match(&message.text)
    }

    fn stop_processing(&self) -> bool {
        true
    }
}
