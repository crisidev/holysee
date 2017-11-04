pub mod client {
    extern crate futures;
    extern crate telegram_bot;
    extern crate tokio_core;


    use std::thread::JoinHandle;
    use self::futures::Stream;
    use settings::Settings;
    use message::{Message, TransportType};
    use std::thread;
    use std::sync::mpsc::{channel, Sender};
    use self::telegram_bot::Api;
    use self::telegram_bot::types::{ChatId, MessageKind, SendMessage, UpdateKind, Update};
    use self::tokio_core::reactor::Core;

    pub struct TelegramClient {
        pub writer: Sender<Message>,
        core: Option<Core>,
        token: String,
        chat_id: i64,
        future: Option<String>,
    }

    impl TelegramClient {
        fn handle_update(&mut self, update: Update) {
            match update.kind {
                UpdateKind::Message(m) => match m.kind {
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
                        self.writer
                            .send(Message {
                                transport: TransportType::Telegram,
                                from: from,
                                to: String::from("-"),
                                text: data,
                            })
                            .unwrap()
                    }
                    _ => {
                        debug!("messageKind != text");
                    }
                },
                _ => {
                    debug!("UpdateKind != messate");
                }
            }
        }

        pub fn new(settings: &Settings, writer: Sender<Message>) -> TelegramClient {
            let (self_writer, reader) = channel::<Message>();

            info!("Created telegram client");
            debug!("Running from configuration: {:?}", settings);

            let token = settings.telegram.token.clone();
            let chat_id = settings.telegram.chat_id;

            thread::spawn(move || {
                let mut core = Core::new().unwrap();
                let api = Api::configure(&token).build(core.handle());
                let chat = ChatId::new(chat_id);
                loop {
                    match reader.recv() {
                        Ok(msg) => {
                            core.run(api.send(SendMessage::new(
                                chat,
                                format!("{}: {}", msg.from, msg.text),
                            ))).unwrap();
                        }
                        Err(e) => {
                            info!("Error reading from internal channel: {}", e);
                        }
                    };
                }
            });

            let mut core = Core::new().unwrap();
            let api = Api::configure(&self.token).build(core.handle());
            let future = api.stream().for_each(|update| {
                self.handle_update(update);
                Ok(())
            });

            let tg = TelegramClient {
                core,
                writer,
                token,
                chat_id,
            };

            tg.future = api.stream().for_each(|update| {
                tg.handle_update(update);
                Ok(())
            });

            return tg
        }

        pub fn start(&mut self) {
            thread::spawn(move || {
                self.core.run(self.future)
            });
        }

        pub fn stop(self: &mut Self) {}
    }
}
