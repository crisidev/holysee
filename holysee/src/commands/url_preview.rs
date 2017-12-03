extern crate regex;
extern crate reqwest;
extern crate select;

use std::io::Read;
use std::thread;

use self::select::document::Document;
use self::select::predicate::Name;
use chan::Sender;

use message::{Message, TransportType, DestinationType};
use commands::command_dispatcher::Command;

#[derive(Debug)]
pub struct UrlPreviewCommand {}

impl UrlPreviewCommand {
    pub fn new() -> UrlPreviewCommand {
        UrlPreviewCommand {}
    }

    fn get(
        url: &str,
        destination: &DestinationType,
        from: &str,
        to_irc: &Sender<Message>,
        to_telegram: &Sender<Message>,
    ) {
        let result = reqwest::get(url);
        match result {
            Ok(mut resp) => {
                let mut buf = String::new();
                match resp.read_to_string(&mut buf) {
                    Ok(n) => debug!("Preview for {} size {}", url, n),
                    Err(e) => error!("Error reading data from {}: {}", url, e),
                }
                let document = Document::from(buf.as_ref());
                for node in document.find(Name("title")) {
                    println!("{:#?}", node);
                    let destination_inner = match *destination {
                        DestinationType::Channel(ref c) => DestinationType::Channel(c.clone()),
                        DestinationType::User(_) => DestinationType::User(from.to_string()),
                        DestinationType::Unknown => {
                            panic!("Serious bug in url_preview command handler")
                        }
                    };
                    node.children().for_each(move |x| {
                        // SEND MESSAGES
                        let title_irc = x.as_text().unwrap();
                        debug!("Extracted url: {}", title_irc);
                        let title_telegram = title_irc;
                        let destination_irc: DestinationType =
                            DestinationType::klone(&destination_inner);
                        let destination_telegram: DestinationType =
                            DestinationType::klone(&destination_inner);
                        to_irc.send(Message::new(
                            TransportType::Telegram,
                            title_irc.to_owned(),
                            String::from("UrlPreviewCommand"),
                            destination_irc,
                            true,
                        ));
                        to_telegram.send(Message::new(
                            TransportType::IRC,
                            title_telegram.to_owned(),
                            String::from("UrlPreviewCommand"),
                            destination_telegram,
                            true,
                        ));
                    });
                }
            }
            Err(e) => {
                error!("Error previewing: {}", e);
            }
        };
    }
}

impl Command for UrlPreviewCommand {
    fn execute(
        &mut self,
        msg: &mut Message,
        to_irc: &Sender<Message>,
        to_telegram: &Sender<Message>,
    ) {
        let re = regex::Regex::new(
            r"(https?://(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,6}\b(?:[-a-zA-Z0-9@:;%()\[\]{}_\+.*~#?,&//=]*))",
        ).unwrap();

        // COMMAND HANDLING
        let message_text = msg.text.to_owned();
        for cap in re.captures_iter(&message_text) {
            let url = String::from(&cap[1]);
            let to_irc_clone = to_irc.clone();
            let to_telegram_clone = to_telegram.clone();
            let destination: DestinationType = DestinationType::klone(&msg.to);
            let from: String = msg.from.clone();
            debug!("Previewing url {}", url);
            thread::spawn(move || {
                UrlPreviewCommand::get(&url, &destination, &from, &to_irc_clone, &to_telegram_clone)
            });
        }
    }

    fn get_usage(&self) -> String {
        String::from(
            "This command is not a real command, therefore it has no usage",
        )
    }

    fn get_name(&self) -> String {
        String::from("url_preview")
    }

    fn matches_message_text(&self, _: &Message) -> bool {
        true
    }

    fn stop_processing(&self) -> bool {
        false
    }
}
