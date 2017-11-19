#![cfg_attr(feature = "clippy", feature(plugin))]
#![cfg_attr(feature = "clippy", plugin(clippy))]
extern crate config;
#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate futures;
extern crate tokio_core;
#[macro_use]
extern crate chan;

mod ircclient;
mod telegram;
mod settings;
mod message;
mod commands;

use std::process;
use settings::Settings;
use message::Message;
use commands::{CommandDispatcher, MessageAsCommand, SendToTelegramCommand, SendToIrcCommand,
               KarmaCommand};

fn main() {
    pretty_env_logger::init().unwrap();

    let settings = match Settings::new() {
        Ok(s) => s,
        Err(e) => {
            error!("Error accessing config file: {}", e);
            process::exit(1)
        }
    };

    // TODO: fix this hardcoded value
    let (to_irc, from_irc) = chan::sync(100);
    let (to_telegram, from_telegram) = chan::sync(100);

    let irc_client = ircclient::client::new(&settings, to_irc.clone());
    let telegram_client = telegram::client::new(&settings, to_telegram.clone());

    info!("Starting up");

    let mut command_dispatcher = CommandDispatcher::new();

    loop {
        let current_message: Message;
        chan_select! {
            from_irc.recv() -> irc_answer => {
                match irc_answer {
                    Some(msg) => {
                        current_message = msg;
                    },
                    None => {
                        error!("Channel disconnected!");
                        continue
                    },
                };
            },
            from_telegram.recv() -> telegram_answer => {
                match telegram_answer {
                    Some(msg) => {
                        current_message = msg;
                    },
                    None => {
                        error!("Channel disconnected!");
                        continue
                    },
                };
            }
        }

        let message_as_command = MessageAsCommand::new();
        let send_to_irc_command = SendToIrcCommand::new(message_as_command);
        let send_to_telegram_command = SendToTelegramCommand::new(message_as_command);
        let karma_command = KarmaCommand::new(message_as_command);

        if karma_command
            .matches_message_text(&current_message.text)
            .is_some()
        {
            debug!("karma evaluation");
            command_dispatcher.set_command(Box::new(karma_command));
            command_dispatcher.execute(&current_message, &irc_client, &telegram_client);
        } else if send_to_telegram_command
                   .matches_message_text(&current_message.text)
                   .is_some() || settings.telegram.allow_receive
        {
            debug!("send to TELEGRAM");
            command_dispatcher.set_command(Box::new(send_to_telegram_command));
            command_dispatcher.execute(&current_message, &irc_client, &telegram_client);
        } else if send_to_irc_command
                   .matches_message_text(&current_message.text)
                   .is_some() || settings.irc.allow_receive
        {
            debug!("send to IRC");
            command_dispatcher.set_command(Box::new(send_to_irc_command));
            command_dispatcher.execute(&current_message, &irc_client, &telegram_client);
        } else {
            debug!("unknown message");
        }
    }
}
