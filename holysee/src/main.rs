#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate config;
extern crate pretty_env_logger;
extern crate telebot;
extern crate tokio_core;
extern crate futures;
extern crate rand;

mod ircclient;
mod telegram;
mod error;
mod settings;
mod message;

use std::process;
use settings::Settings;
use message::{Message, TransportType};
use rand::Rng;


use std::sync::mpsc::TryRecvError;
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

    let irc = ircclient::client::new(&settings, sender_for_irc.clone());

    let mut diocane = 0;
    let mut rng = rand::thread_rng();
    loop {
        diocane = diocane + 1;
        match from_irc.try_recv() {
            Ok(msg) => debug!("from irc: {}", msg.text),
            Err(TryRecvError::Empty) => {
            },
            Err(TryRecvError::Disconnected) => {
                error!("Channel disconnected!");
            },
        };

        let doit = rng.gen_range(0.0, 1.0);
        if  doit < 0.0000001 {
            irc.send(Message {
                from: String::from("telegram_user_sender"),
                text: String::from(format!("generated_message: {}", doit)),
                to: String::from("telegram_channel_name"),
                transport: TransportType::Telegram,
            }).unwrap();
        }
    }
}
