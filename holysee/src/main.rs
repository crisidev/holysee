#![feature(mpsc_select)]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
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
        Err(_) => {
            error!("Error accessing config file");
            process::exit(1)
        }
    };

    let (sender_for_irc, from_irc) = mpsc::channel::<Message>();
    let (sender_for_tg, from_tg) = mpsc::channel::<Message>();

    let irc = ircclient::client::new(&settings, sender_for_irc.clone());
    let tg = telegram::client::new(&settings, sender_for_tg.clone());

    info!("Starting up");
    loop {
        select! {
            irc_answer = from_irc.recv() => {
                match irc_answer {
                    Ok(msg) => {tg.send(msg).unwrap()},
                    Err(RecvError) => {
                        error!("Channel disconnected!");
                    },
                };
            },
            tg_answer = from_tg.recv() => {
                match tg_answer {
                    Ok(msg) => irc.send(msg).unwrap(),
                    Err(RecvError) => {
                        error!("Channel disconnected!");
                    }
                }
            }
        }
    }
}
