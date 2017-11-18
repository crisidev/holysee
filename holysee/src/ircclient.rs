pub mod client {
    extern crate irc;

    use std::default::Default;
    use self::irc::client::prelude::*;
    use settings::Settings;
    use message::{Message, TransportType};
    use std::thread;
    use std::sync::mpsc::{channel, Sender};


    pub fn new(settings: &Settings, to_int_sender_obj: Sender<Message>) -> Sender<Message> {
        let (return_sender, from_int_reader) = channel::<Message>();
        let cfg = Config {
            nickname: Some(settings.irc.nickname.to_owned()),
            username: Some(settings.irc.username.to_owned()),
            realname: Some(settings.irc.real_name.to_owned()),
            nick_password: Some(settings.irc.password.to_owned()),
            server: Some(settings.irc.host.to_owned()),
            port: Some(settings.irc.port.to_owned()),
            channels: Some(vec![settings.irc.channel.to_owned()]),
            use_ssl: Some(settings.irc.ssl.to_owned()),
            ..Default::default()
        };
        let channel_to_send = settings.irc.channel.clone();
        info!(
            "Created irc client for {}:{}",
            settings.irc.host,
            settings.irc.port
        );
        debug!("Running from configuration: {:?}", settings);
        let the_server = IrcServer::from_config(cfg).unwrap();
        the_server.identify().unwrap();
        let the_server_clone = the_server.clone();
        let username_clone = settings.irc.username.clone();
        let channel_clone = settings.irc.channel.clone();
        thread::spawn(move || {
            the_server_clone.for_each_incoming(|m| {
                let srcnick = match m.source_nickname() {
                    Some(x) => String::from(x),
                    None => String::from("undefined"),
                };
                match m.command {
                    irc::proto::Command::PRIVMSG(source, message_text) => {
                        debug!(
                            "Incoming message source: {} message_text: {} srcnick: {}",
                            source,
                            message_text,
                            srcnick
                        );
                        to_int_sender_obj
                            .send(Message {
                                from_transport: TransportType::IRC,
                                from: srcnick,
                                to: source,
                                text: message_text,
                            })
                            .unwrap()
                    }
                    irc::proto::Command::INVITE(_, channel) => {
                        debug!("got invite for channel: {}", channel);
                        if channel == channel_clone {
                            debug!("chosen to join channel {}", channel);
                            the_server_clone.send_join(&channel).unwrap()
                        }
                    }
                    irc::proto::Command::NOTICE(_, notice) => {
                        debug!("NOTICE: {}", notice);
                        if notice.contains("You are now identified for") {
                            debug!("identified successfully for {}", username_clone);
                            the_server_clone.send_join(&channel_clone).unwrap()
                        }
                    }
                    irc::proto::Command::MOTD(_) => {}
                    _ => debug!("irc message:  {:#?}", m),
                }
            })
        });

        thread::spawn(move || loop {
            match from_int_reader.recv() {
                Ok(msg) => {
                    match the_server.send_privmsg(&channel_to_send, msg.format().as_ref()) {
                        Ok(_) => {
                            info!("Message sent");
                        }
                        Err(_) => {
                            info!("Could not send, server disconnected");
                        }
                    }
                }
                Err(e) => {
                    info!("Error reading from internal channel: {}", e);
                }
            };
        });
        return_sender.clone()
    }
}
