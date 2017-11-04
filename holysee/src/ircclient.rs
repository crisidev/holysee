pub mod client {
    extern crate irc;

    use std::default::Default;
    use self::irc::client::prelude::*;
    use settings::Settings;
    use message::{Message, TransportType};
    use std::thread;
    use std::sync::mpsc::{Sender, channel};


    pub fn new(settings: &Settings, to_int_sender_obj: Sender<Message>) -> Sender<Message> {
        let (return_sender, from_int_reader) = channel::<Message>();
        let cfg = Config {
            nickname: Some(settings.irc.nickname.to_owned()),
            username: Some(settings.irc.username.to_owned()),
            realname: Some(settings.irc.real_name.to_owned()),
            nick_password: Some(settings.irc.password.to_owned()),
            server: Some(settings.irc.host.to_owned()),
            port: Some(settings.irc.port.to_owned()),
            channels: Some(vec!(settings.irc.channel.to_owned())),
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
        let the_server_clone = the_server.clone();
        the_server.identify().unwrap();
        thread::spawn(move || {
            the_server_clone.for_each_incoming(|m| {
                match m.command {
                    irc::proto::Command::PRIVMSG(source, message_text) =>
                        to_int_sender_obj.send(Message {
                            transport: TransportType::IRC,
                            from: source,
                            to: String::from("-"),
                            text: message_text,
                        }).unwrap(),
                    _ => {},
                }
            })
        });

        thread::spawn(move || {
            loop {
                match from_int_reader.recv() {
                    Ok(msg) => match the_server.send_privmsg(&channel_to_send, msg.text.as_ref()) {
                        Ok(_) => {
                            info!("message sent");
                        }
                        Err(_) => {
                            info!("could not send, server disconnected");
                        }
                    },
                    Err(e) => {
                        info!("Error reading from internal channel: {}", e);
                    }
                };
            }
        });
        return_sender.clone()
    }
}