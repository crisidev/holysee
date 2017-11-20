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
use commands::{CommandDispatcher, RelayMessageCommand, KarmaCommand, LastSeenCommand};

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

    info!("Starting Holysee");

    let mut command_dispatcher = CommandDispatcher::new(&settings.commands);

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

        debug!("Current message: {:#?}", current_message);

        let relay_command = RelayMessageCommand::new(
            &settings.irc.allow_receive,
            &settings.telegram.allow_receive,
            &settings.command_prefix,
        );
        let karma_command = KarmaCommand::new(&settings.command_prefix, &settings.commands);
        let last_seen_command = LastSeenCommand::new(&settings.command_prefix, &settings.commands);

        if command_dispatcher.is_command_enabled(&karma_command.name) &&
            karma_command.matches_message_text(&current_message)
        {
            command_dispatcher.set_command(Box::new(karma_command));
            command_dispatcher.execute(&current_message, &to_irc, &to_telegram);
        } else if command_dispatcher.is_command_enabled(&relay_command.name) &&
                   relay_command.matches_message_text(&current_message)
        {
            command_dispatcher.set_command(Box::new(relay_command));
            command_dispatcher.execute(&current_message, &irc_client, &telegram_client);
        } else if command_dispatcher.is_command_enabled(&last_seen_command.name) &&
            last_seen_command.matches_message_text(&current_message) {
            command_dispatcher.set_command(Box::new(last_seen_command));
            command_dispatcher.execute(&current_message, &irc_client, &telegram_client);
        }
    }
}
