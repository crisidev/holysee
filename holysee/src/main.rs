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

mod ircclient;
mod error;
mod settings;

use std::process;
use std::thread;
use telebot::bot;
use tokio_core::reactor::Core;
use futures::stream::Stream;
use futures::Future;
use settings::Settings;

use telebot::functions::*;

use std::sync::mpsc::{Sender, Receiver};
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

    let irc = match ircclient::IrcClient::new(&settings) {
        Ok(s) => s,
        Err(_) => {
            error!("Error creating the irc client");
            process::exit(1);
        }
    };

    let (tx, rx) = mpsc::channel::<String>();

    // let mut lp = Core::new().unwrap();
    // let bot = bot::RcBot::new(lp.handle(), settings.telegram.token.as_ref()).update_interval(200);
    //
    // let handle = bot.new_cmd("/reply").and_then(|(bot, msg)| {
    //     let mut text = msg.text.unwrap().clone();
    //     if text.is_empty() {
    //         text = "<empty>".into();
    //     }
    //
    //     bot.message(msg.chat.id, text).send()
    // });
    //
    // bot.register(handle);

    let irc_builder = thread::Builder::new().name("irc".into());
    let irc_thread = irc_builder.spawn(move || irc.run(tx)).unwrap();

    loop {
        let message = rx.recv().unwrap();
        info!("message on queue: {:?}", message);
    }

    // bot.run(&mut lp).unwrap();
    irc_thread.join().unwrap();
}
