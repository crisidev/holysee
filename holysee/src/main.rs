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
use std::collections::HashMap;
use settings::Settings;
use message::Message;
use commands::command_dispatcher::Command;
use commands::last_seen::LastSeenCommand;
use commands::relay::RelayMessageCommand;
use commands::karma::KarmaCommand;
use commands::command_dispatcher::CommandDispatcher;
use commands::quote::QuoteCommand;
use commands::url_preview::UrlPreviewCommand;
use commands::usage::UsageCommand;

fn main() {
    pretty_env_logger::init().unwrap();
    let mut usage_hashmap: HashMap<String, String> = HashMap::new();
    let settings = match Settings::new(true) {
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

    let mut karma_command = KarmaCommand::new(&settings.command_prefix, &settings.commands, true);
    let mut last_seen_command =
        LastSeenCommand::new(&settings.command_prefix, &settings.commands, true);
    let mut quote_command = QuoteCommand::new(&settings.command_prefix, &settings.commands, true);
    let mut url_preview_command = UrlPreviewCommand::new(true);
    let mut relay_command = RelayMessageCommand::new(
        &settings.irc.allow_receive,
        &settings.telegram.allow_receive,
        &settings.command_prefix,
        true,
    );
    usage_hashmap.insert(
        karma_command.get_name().clone(),
        karma_command.get_usage().clone(),
    );
    usage_hashmap.insert(
        quote_command.get_name().clone(),
        quote_command.get_usage().clone(),
    );
    usage_hashmap.insert(
        last_seen_command.get_name().clone(),
        last_seen_command.get_usage().clone(),
    );
    usage_hashmap.insert(
        url_preview_command.get_name().clone(),
        url_preview_command.get_usage().clone(),
    );
    let mut usage_command = UsageCommand::new(&settings.command_prefix, &mut usage_hashmap, true);
    let mut command_dispatcher = CommandDispatcher::new();

    // FILTERS
    command_dispatcher.register(&mut last_seen_command);
    command_dispatcher.register(&mut url_preview_command);
    // COMMANDS
    // karma command
    command_dispatcher.register(&mut karma_command);
    // quote command
    command_dispatcher.register(&mut quote_command);
    // relay command
    command_dispatcher.register(&mut relay_command);
    // usage command
    command_dispatcher.register(&mut usage_command);

    loop {
        let current_message: Message;
        chan_select! {
            from_irc.recv() -> irc_answer => {
                match irc_answer {
                    Some(msg) => {
                        debug!("Received one message from IRC chan");
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
                        debug!("Received one message from telegram chan");
                        current_message = msg;
                    },
                    None => {
                        error!("Channel disconnected!");
                        continue
                    },
                };
            }
        }

        debug!("Current HolySee message: {:#?}", current_message);
        command_dispatcher.execute(&current_message, &irc_client, &telegram_client);

    }
}
