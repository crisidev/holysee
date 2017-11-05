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

use std::process;
use settings::Settings;
use message::Message;

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
        if current_message.is_command(settings.command_prefix.as_ref()) {
            current_message.handle_command(irc_sender.clone(), tg_sender.clone());
        } else {
            match current_message.from_transport {
                message::TransportType::Telegram => {
                    if settings.irc.allow_receive {
                        irc_sender.send(current_message).unwrap();
                    }
                }
                message::TransportType::IRC => {
                    if settings.telegram.allow_receive {
                        tg_sender.send(current_message).unwrap();
                    }
                }
            };
        }
    }
}
