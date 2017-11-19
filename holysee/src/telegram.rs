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
                    core.run(api.send(SendMessage::new(chat, msg.format())))
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
                            to_main_queue.send(Message {
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
