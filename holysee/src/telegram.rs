pub mod client {
    extern crate futures;
    extern crate telegram_bot;
    extern crate tokio_core;
    extern crate chan;

    use std::thread;
    use chan::{Sender, Receiver};

    use self::futures::Stream;
    use self::telegram_bot::Api;
    use self::telegram_bot::types::{ChatId, MessageKind, SendMessage, UpdateKind};
    use self::tokio_core::reactor::Core;

    use settings::Settings;
    use message::{Message, TransportType};

    fn main_to_telegram_loop(from_main_queue: &Receiver<Message>, token: &str, chat_id: i64) {
        let mut core = Core::new().unwrap();
        let api = Api::configure(token).build(core.handle());
        let chat = ChatId::new(chat_id);
        loop {
            let current: Option<Message> = from_main_queue.recv();
            match current {
                Some(msg) => {
                    match core.run(api.send(SendMessage::new(chat, msg.text.as_ref()))) {
                        Ok(_) => info!("Telegram message sent"),
                        Err(e) => error!("Sending message failed for reason: {:#?}", e),
                    };
                }
                None => {
                    info!("No message to read on internal channel");
                }
            };
        }
    }

    fn telegram_to_main_loop(to_main_queue: &Sender<Message>, token: &str) {
        loop {
            let mut core = Core::new().unwrap();
            let api = Api::configure(token).build(core.handle());
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
                                debug!("Incoming Telegram message source: #cattedrale, text: {}, src_nick: {}, entities: {:?}", data, from, entities);
                                to_main_queue.send(Message::new(
                                    TransportType::Telegram,
                                    data,
                                    from,
                                    String::from("-"),
                                    false,
                                ));
                            }
                            _ => {
                                debug!("Telegram message type != text");
                            }
                        }
                    }
                    _ => {
                        debug!("Telegram update type != message");
                    }
                }
                Ok(())
            });
            match core.run(future) {
                Ok(item) => debug!("Item from core.run: {:#?}", item),
                Err(e) => error!("Main loop exiting for reason: {:#?}", e),
            }
            warn!("Restarting message receive loop");
        }
    }

    pub fn new(settings: &Settings, to_main_queue: Sender<Message>) -> Sender<Message> {
        // TODO fix this hardcoded value
        let (to_telegram_queue, from_main_queue) = chan::sync(100);

        info!("Created telegram client");
        debug!("Running from configuration: {:?}", settings);

        let token = settings.telegram.token.clone();
        let token_clone = settings.telegram.token.clone();
        let chat_id = settings.telegram.chat_id;

        thread::spawn(move || telegram_to_main_loop(&to_main_queue, &token));
        thread::spawn(move || {
            main_to_telegram_loop(&from_main_queue, &token_clone, chat_id)
        });

        to_telegram_queue.clone()
    }
}
