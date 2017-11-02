#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate config;
extern crate pretty_env_logger;
extern crate telebot;
extern crate tokio_core;
extern crate futures;

mod error;
mod settings;

use telebot::bot;
use tokio_core::reactor::Core;
use futures::stream::Stream;
use futures::Future;
use std::fs::File;
use settings::Settings;
use std::process;

use telebot::functions::*;

fn main() {
    pretty_env_logger::init().unwrap();

    let settings = match Settings::new() {
        Ok(s) => s,
        Err(_) => {
            error!("Error accessing config file");
            process::exit(1);
        },
    };

let mut lp = Core::new().unwrap();
    let bot = bot::RcBot::new(lp.handle(), settings.telegram.token.as_ref())
        .update_interval(200);

    let handle = bot.new_cmd("/reply")
        .and_then(|(bot, msg)| {
            let mut text = msg.text.unwrap().clone();
            if text.is_empty() {
                text = "<empty>".into();
            }

            bot.message(msg.chat.id, text).send()
        });

    bot.register(handle);

    bot.run(&mut lp).unwrap();
}
