pub mod client {
    extern crate irc;
    extern crate chan;

    use std::thread;
    use std::process;
    use std::default::Default;
    use chan::{Sender, Receiver};

    use self::irc::client::prelude::*;

    use settings::Settings;
    use message::{Message, TransportType};

    fn main_to_irc_loop(
        from_main_queue: &Receiver<Message>,
        server: &IrcServer,
        channel_name: &str,
    ) {
        loop {
            let current: Option<Message> = from_main_queue.recv();
            match current {
                Some(msg) => {
                    if msg.is_from_command {
                        match server.send_notice(channel_name, msg.text.as_ref()) {
                            Ok(_) => {
                                info!("IRC NOTICE sent");
                            }
                            Err(_) => {
                                error!("Could not send, server disconnected");
                            }
                        }
                    } else {
                        match server.send_privmsg(channel_name, msg.text.as_ref()) {
                            Ok(_) => {
                                info!("IRC PRIVMSG sent");
                            }
                            Err(_) => {
                                error!("Could not send, server disconnected");
                            }
                        }
                    }
                }
                None => {
                    info!("No message for reading on internal channel");
                }
            };
        }
    }

    fn irc_to_main_loop(to_main_queue: &Sender<Message>, server: &IrcServer, channel_name: &str) {
        loop {
            match server.for_each_incoming(|m| {
                let srcnick = match m.source_nickname() {
                    Some(x) => String::from(x),
                    None => String::from("undefined"),
                };
                match m.command {
                    irc::proto::Command::PRIVMSG(source, message_text) => {
                        debug!(
                            "Incoming IRC message source: {}, text: {}, src_nick: {}",
                            source,
                            message_text,
                            srcnick
                        );
                        to_main_queue.send(Message::new(
                            TransportType::IRC,
                            message_text,
                            srcnick,
                            source,
                            false,
                        ));
                    }
                    irc::proto::Command::INVITE(_, channel) => {
                        debug!("Got invite for channel: {}", channel);
                        if channel == channel_name {
                            debug!("Chosen to join channel {}", channel);
                            server.send_join(&channel).unwrap();
                        }
                    }
                    irc::proto::Command::NOTICE(_, notice) => {
                        debug!("NOTICE: {}", notice);
                        if notice.contains("You are now identified for") {
                            debug!("Identified successfully");
                            server.send_join(channel_name).unwrap();
                        }
                    }
                    irc::proto::Command::MOTD(_) => {}
                    _ => debug!("IRC message:  {:#?}", m),
                };
            }) {
                Ok(item) => debug!("Item from server.for_each_incoming: {:#?}", item),
                Err(e) => error!("Server loop exiting for reason: {:#?}", e),
            };
            warn!("Restarting message receive loop");
        }
    }


    pub fn new(settings: &Settings, to_main_queue: Sender<Message>) -> Sender<Message> {
        // TODO: fix this hardcoded value
        let (to_irc_queue, from_main_queue) = chan::sync(100);
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
        info!(
            "Created irc client for {}:{}",
            settings.irc.host,
            settings.irc.port
        );
        debug!("Running from configuration: {:?}", settings);
        let irc_to_main_server = IrcServer::from_config(cfg).unwrap();
        match irc_to_main_server.identify() {
            Ok(_) => {
                info!("Connection successfull");
            }
            Err(e) => {
                error!("IRC server error: {}", e);
                process::exit(1);
            }
        };
        let main_to_irc_server = irc_to_main_server.clone();
        let irc_channel_name = settings.irc.channel.clone();
        let irc_channel_name_clone = settings.irc.channel.clone();

        thread::spawn(move || {
            irc_to_main_loop(&to_main_queue, &irc_to_main_server, &irc_channel_name)
        });
        thread::spawn(move || {
            main_to_irc_loop(
                &from_main_queue,
                &main_to_irc_server,
                &irc_channel_name_clone,
            )
        });

        to_irc_queue.clone()
    }
}
