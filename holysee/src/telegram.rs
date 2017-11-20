pub mod client {
    extern crate futures;
    extern crate telegram_bot;
    extern crate tokio_core;
    extern crate chan;

    use self::futures::Stream;
    use settings::Settings;
    use message::{Message, TransportType};
    use std::thread;
    use chan::{Sender, Receiver};
    use self::telegram_bot::Api;
    use self::telegram_bot::types::{ChatId, MessageKind, SendMessage, UpdateKind};
    use self::tokio_core::reactor::Core;

    fn main_to_telegram_loop(from_main_queue: &Receiver<Message>, token: &String, chat_id: i64) {
        let mut core = Core::new().unwrap();
        let api = Api::configure(token).build(core.handle());
        let chat = ChatId::new(chat_id);
        loop {
            let current: Option<Message> = from_main_queue.recv();
            match current {
                Some(msg) => {
                    core.run(api.send(SendMessage::new(chat, msg.text.as_ref())))
                        .unwrap();
                }
                None => {
                    info!("No message to read on internal channel");
                }
            };
        }
    }

    fn telegram_to_main_loop(to_main_queue: &Sender<Message>, token: &String) {
        let mut core = Core::new().unwrap();
        let api = Api::configure(&token).build(core.handle());
        let future = api.stream().for_each(|update| {
            match update.kind {
                UpdateKind::Message(m) => {
                    match m.kind {
                        MessageKind::Text { data, entities } => {
                            let from: String = match m.from {
                                // user is present, check its fields
                                Some(u) => {
                                    match u.username {
                                        // if username is provided, use it
                                        Some(username) => username,
                                        // if username is not provided, try telegram profile names
                                        None => {
                                            // first_name always contains something
                                            match u.last_name {
                                                Some(last_name) => {
                                                    format!("{} {}", u.first_name, last_name)
                                                }
                                                None => u.first_name,
                                            }
                                        }
                                    }
                                }
                                // user is not present, should never happen
                                None => String::from("unset"),
                            };
                            debug!("entities: {:#?} from: {}", entities, from);
                            to_main_queue.send(Message::new(
                                TransportType::Telegram,
                                data,
                                from,
                                String::from("-"),
                                false,
                            ));
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
        core.run(future).unwrap();
    }

    pub fn new(settings: &Settings, to_main_queue: Sender<Message>) -> Sender<Message> {
        // TODO fix this hardcoded value
        let (to_telegram_queue, from_main_queue) = chan::sync(100);

        info!("Created telegram client");
        debug!("Running from configuration: {:?}", settings);

        let token = settings.telegram.token.clone();
        let token_clone = settings.telegram.token.clone();
        let chat_id = settings.telegram.chat_id.clone();

        thread::spawn(move || telegram_to_main_loop(&to_main_queue, &token));
        thread::spawn(move || {
            main_to_telegram_loop(&from_main_queue, &token_clone, chat_id)
        });

        to_telegram_queue.clone()
    }
}
