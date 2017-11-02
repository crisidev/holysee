#[macro_use] extern crate log;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate config;
extern crate pretty_env_logger;

mod ircclient;
mod error;
mod settings;

use settings::Settings;
use std::process;

fn main() {
    pretty_env_logger::init().unwrap();

    let settings = match Settings::new() {
        Ok(s) => s,
        Err(_) => {
            error!("Error accessing config file");
            process::exit(1);
        },
    };

    let irc = match ircclient::IrcClient::new(&settings) {
        Ok(s) => s,
        Err(_) => {
            error!("Error creating the irc client");
            process::exit(1);
        },
    };
    irc.run();
//    thread::sleep(time::Duration::from_secs(30));
}
