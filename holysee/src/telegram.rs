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