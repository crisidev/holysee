extern crate config;
extern crate serde;

#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
extern crate pretty_env_logger;


mod settings;

use settings::Settings;
use std::process;

fn main() {
    pretty_env_logger::init().unwrap();

    let settings = match Settings::new() {
        Ok(s) => s,
        Err(err) => {
            error!("Error accessing config file");
            process::exit(1);
        },
    };
}
