pub mod client {
    extern crate futures;
    extern crate telegram_bot;
    extern crate tokio_core;
    extern crate chan;

    use self::futures::Stream;
    use settings::Settings;
    use message::{Message, TransportType};
    use std::thread;
    use chan::{Sender};
    use self::telegram_bot::Api;
    use self::telegram_bot::types::{ChatId, MessageKind, SendMessage, UpdateKind};
    use self::tokio_core::reactor::Core;

    pub fn new(settings: &Settings, to_int_sender_obj: Sender<Message>) -> Sender<Message> {
        // TODO fix this hardcoded value
        let (return_sender, from_int_reader) = chan::sync(100);

        info!("Created telegram client");
        debug!("Running from configuration: {:?}", settings);

        let token = settings.telegram.token.clone();


        thread::spawn(move || {
            let mut core = Core::new().unwrap();
            let api = Api::configure(&token).build(core.handle());
            let future = api.stream().for_each(|update| {
                match update.kind {
                    UpdateKind::Message(m) => {
                        match m.kind {
                            MessageKind::Text { data, entities } => {
                                let from: String = m.from
                                    .clone()
                                    .and_then(|u| {
                                        u.username.clone().or_else(|| {
                                            u.last_name
                                            .clone()
                                            .and_then(
                                                |ln| Some(format!("{:?} {:?}", u.first_name, ln)),
                                            )
                                            .or_else(|| Some(u.first_name.clone()))
                                        })
                                    })
                                    .unwrap_or(String::from("unset"));
                                debug!("entities: {:#?} from: {}", entities, from);
                                to_int_sender_obj
                                    .send(Message {
                                        from_transport: TransportType::Telegram,
                                        from: from,
                                        to: String::from("-"),
                                        text: data,
                                    })
                            }
                            _ => {
                                debug!("messageKind != text");
                            }
                        }
                    }
                    _ => {
                        debug!("UpdateKind != messate");
                    }
                }
                Ok(())
            });
            core.run(future)
        });

        let chat_id = settings.telegram.chat_id;
        let token_clone = settings.telegram.token.clone();

        thread::spawn(move || {
            let mut core = Core::new().unwrap();
            let api = Api::configure(&token_clone).build(core.handle());
            let chat = ChatId::new(chat_id);
            loop {
                let current: Option<Message> = from_int_reader.recv();
                match  current {
                    Some(msg) => {
                        core.run(api.send(SendMessage::new(chat, msg.format())))
                            .unwrap();
                    }
                    None => {
                        info!("No message to read on internal channel");
                    }
                };
            }
        });
        return_sender.clone()
    }
}
