pub mod client {
    extern crate telegram_bot;
    extern crate tokio_core;
    extern crate futures;

    use self::futures::Stream;
    use settings::Settings;
    use message::{Message, TransportType};
    use std::thread;
    use std::sync::mpsc::{Sender, channel};
    use self::telegram_bot::Api;
    use self::telegram_bot::types::{UpdateKind, MessageKind, ChatId, SendMessage};
    use self::tokio_core::reactor::Core;

    pub fn new(settings: &Settings, to_int_sender_obj: Sender<Message>) -> Sender<Message> {
        let (return_sender, from_int_reader) = channel::<Message>();

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
                            MessageKind::Text{data,entities} => {
                                to_int_sender_obj.send(Message {
                                    transport: TransportType::Telegram,
                                    from: m.from.unwrap().username.unwrap(),
                                    to: String::from("-"),
                                    text: data,
                                }).unwrap()
                            },
                            _ => {
                                info!("messageKind != text");
                            },
                        }
                    }
                    _ => {
                        info!("UpdateKind != messate");
                    },
                }
                Ok(())
            });
            core.run(future)
        });

        let chat_id = settings.telegram.chat_id.clone();
        let token_clone = settings.telegram.token.clone();

        thread::spawn(move || {
            let mut core = Core::new().unwrap();
            let api = Api::configure(&token_clone).build(core.handle());
            let chat = ChatId::new(chat_id);
            loop {
                match from_int_reader.recv() {
                    Ok(msg) => {
                        core.run(api.send(SendMessage::new(chat, format!("{}: {}", msg.from, msg.text)))).unwrap();
                    },
                    Err(e) => {
                        info!("Error reading from internal channel: {}", e);
                    },
                };
            }
        });
        return_sender.clone()
    }
}