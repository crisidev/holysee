#![feature(mpsc_select)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate config;
extern crate pretty_env_logger;

extern crate tokio_core;
extern crate futures;

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
            process::exit(1);
        }
    };

    let (sender_for_irc, from_irc) = mpsc::channel::<Message>();
    let (sender_for_tg, from_tg) = mpsc::channel::<Message>();

    let irc = ircclient::client::new(&settings, sender_for_irc.clone());
    let tg = telegram::client::new(&settings, sender_for_tg.clone());

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

