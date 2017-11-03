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

    use settings::Settings;
    use message::{Message, TransportType};
    use std::thread;
    use std::sync::mpsc::{Sender, channel};
    use self::telegram_bot::*;

    pub fn new(settings: &Settings, to_int_sender_obj: Sender<Message>) -> Sender<Message> {
        let (return_sender, from_int_reader) = channel::<Message>();

        info!("Created telegram client");
        debug!("Running from configuration: {:?}", settings);

        let api = Api::from_token(settings.telegram.token.as_ref()).unwrap();
        let api_clone = api.clone();

        thread::spawn(move || {
            let mut listener = api.listener(ListeningMethod::LongPoll(None));
            listener.listen(|u| {
                if let Some(m) = u.message {
                    let name = m.from.first_name + &*m.from.last_name
                        .map_or("".to_string(), |mut n| {
                            n.insert(0, ' ');
                            n
                        });
                    let chat_id = m.chat.id();
                    match m.msg {
                        MessageType::Text(t) => {
                            to_int_sender_obj.send(Message {
                                transport: TransportType::Telegram,
                                from: name,
                                to: format!("{}", chat_id),
                                text: t,
                            }).unwrap();
                        }
                        _ => {}
                    };
                }
                Ok(ListeningAction::Continue)
            })
        });

        let chat_id = settings.telegram.chat_id.clone();

        thread::spawn(move || {
            loop {
                match from_int_reader.recv() {
                    Ok(msg) => match api_clone.send_message(
                        chat_id,
                        msg.text,
                        None, None, None, None) {
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