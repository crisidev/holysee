extern crate irc;

use std::default::Default;
use self::irc::client::prelude::*;
use settings::Settings;
use error::IrcClientError;
use std::result::Result;
use std::sync::mpsc::Sender;

pub struct IrcClient {
    server: IrcServer,
}

impl IrcClient {
    pub fn new(settings: &Settings) -> Result<Self, IrcClientError> {
        let cfg = Config {
            nickname: Some(settings.irc.nickname.to_owned()),
            username: Some(settings.irc.username.to_owned()),
            realname: Some(settings.irc.real_name.to_owned()),
            nick_password: Some(settings.irc.password.to_owned()),
            server: Some(settings.irc.host.to_owned()),
            port: Some(settings.irc.port.to_owned()),
            channels: Some(settings.irc.channels.to_owned()),
            use_ssl: Some(settings.irc.ssl.to_owned()),
            ..Default::default()
        };
        info!(
            "Created irc client for {}:{}",
            settings.irc.host,
            settings.irc.port
        );
        debug!("Running from configuration: {:?}", settings);
        Ok(IrcClient { server: IrcServer::from_config(cfg).unwrap() })
    }

    pub fn run(&self, tx: Sender<String>) {
        self.server.identify().unwrap();
        info!("Identify successfull");
        let thread_tx = tx.clone();
        self.server
            .for_each_incoming(|message| {
                info!("Got: {}", message);
                match thread_tx.send(message.to_string()) {
                    Ok(_) => {
                        info!("sent message");
                        ()
                    }
                    Err(e) => {
                        error!("send error: {:?}", e);
                        ()
                    }
                }

            })
            .unwrap();
    }
}
