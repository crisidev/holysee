#![feature(mpsc_select)]
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

mod ircclient;
mod telegram;
mod settings;
mod message;
mod commands;

use std::process;
use settings::Settings;
use message::Message;
use commands::{CommandDispatcher, SendToTelegramCommand, SendToIrcCommand, MessageAsCommand};

use std::sync::mpsc::RecvError;
use std::sync::mpsc;


fn main() {
    pretty_env_logger::init().unwrap();

    let settings = match Settings::new() {
        Ok(s) => s,
        Err(e) => {
            error!("Error accessing config file: {}", e);
            process::exit(1)
        }
    };

    let (sender_for_irc, from_irc) = mpsc::channel::<Message>();
    let (sender_for_tg, from_tg) = mpsc::channel::<Message>();

    let irc_sender = ircclient::client::new(&settings, sender_for_irc.clone());
    let tg_sender = telegram::client::new(&settings, sender_for_tg.clone());

    info!("Starting up");

    let mut command_dispatcher = CommandDispatcher::new();

    loop {
        let current_message: Message;
        select! {
            irc_answer = from_irc.recv() => {
                match irc_answer {
                    Ok(msg) => {
                        current_message = msg;
                    },
                    Err(RecvError) => {
                        error!("Channel disconnected!");
                        continue
                    },
                };
            },
            tg_answer = from_tg.recv() => {
                match tg_answer {
                    Ok(msg) => {
                        current_message = msg;
                    },
                    Err(RecvError) => {
                        error!("Channel disconnected!");
                        continue
                    },
                };
            }
        }

        let message_as_command = MessageAsCommand::new();
        let send_to_irc_command = SendToIrcCommand::new(message_as_command);
        let send_to_tg_command = SendToTelegramCommand::new(message_as_command);

        if send_to_tg_command
            .matches_message_text(&current_message.text.clone())
            .is_some() || settings.telegram.allow_receive
        {
            debug!("send to TELEGRAM");
            command_dispatcher.set_command(Box::new(send_to_tg_command));
            command_dispatcher.execute(&current_message, irc_sender.clone(), tg_sender.clone());
        } else if send_to_irc_command
                   .matches_message_text(&current_message.text.clone())
                   .is_some() || settings.irc.allow_receive
        {
            debug!("send to IRC");
            command_dispatcher.set_command(Box::new(send_to_irc_command));
            command_dispatcher.execute(&current_message, irc_sender.clone(), tg_sender.clone());
        } else {
            debug!("unknown message");
        }
    }
}
