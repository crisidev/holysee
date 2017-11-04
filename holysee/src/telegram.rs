//extern crate telebot;
//extern crate tokio_core;
//
//use telebot::RcBot;
//use tokio_core::reactor::Core;
//use futures::stream::Stream;
//use futures;
//use error;
//use message::{Message, TransportType};
//use settings::Settings;
//use std::sync::mpsc::Sender;
//
//
//pub struct TelegramClient {
////    bot: tokio_core::reactor::Core,
//}
//
//impl TelegramClient {
////    pub fn new(settings: &Settings) -> Result<Self, error::TelegramClientError> {
////        let mut lp = Core::new().unwrap();
////        Ok(TelegramClient {
////            bot: lp,//Arc::new(RcBot::new(lp.handle(), settings.telegram.token.as_ref()).update_interval(200)),
////        })
////    }
//
//    pub fn run(&self, tx: Sender<Message>, apikey: String) {
//        let mut lp = Core::new().unwrap();
//        let thread_tx = tx.clone();
//        let bot = RcBot::new(lp.handle(), apikey.as_ref()).update_interval(200);
//        let stream = bot.get_stream().and_then(|(_, msg)| {
//            thread_tx.send(Message{
//                text: msg.message.unwrap().text.unwrap(),
//                transport: TransportType::Telegram,
//                from: "androcchia".into(),
//                to: "lesbazza".into(),
//            }).unwrap();
//            futures::done(Ok(()))
//        });
//        lp.run(stream.for_each(|_| Ok(())));
//        ()
//    }
//}
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
    use self::telegram_bot::types::{UpdateKind, MessageKind, ChatId};
    use self::tokio_core::reactor::Core;
    use self::telegram_bot::CanSendMessage;

    pub fn new(settings: &Settings, to_int_sender_obj: Sender<Message>) -> Sender<Message> {
        let (return_sender, from_int_reader) = channel::<Message>();

        info!("Created telegram client");
        debug!("Running from configuration: {:?}", settings);

        let token = settings.telegram.token.clone();

        thread::spawn(move || {
            let core = Core::new().unwrap();
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
                };
                Ok(())
            });
        });

        let chat_id = settings.telegram.chat_id.clone();
        let token_clone = settings.telegram.token.clone();

        thread::spawn(move || {
            let core = Core::new().unwrap();
            let api = Api::configure(&token_clone).build(core.handle());
            loop {
                match from_int_reader.recv() {
                    Ok(msg) => {
                        let chat = ChatId::new(chat_id);
                        api.spawn(chat.text(msg.text));
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