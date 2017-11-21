extern crate regex;
extern crate reqwest;
extern crate select;

use std::io::Read;

use self::select::document::Document;
use self::select::predicate::Name;
use chan::Sender;

use message::{Message, TransportType};
use commands::command_dispatcher::Command;

#[derive(Debug)]
pub struct UrlPreviewCommand {
    pub name: String,
}

impl UrlPreviewCommand {
    pub fn new() -> UrlPreviewCommand {
        UrlPreviewCommand { name: String::from("url_preview") }
    }
}

impl Command for UrlPreviewCommand {
    fn execute(&mut self, msg: &Message, to_irc: &Sender<Message>, to_telegram: &Sender<Message>) {
        info!("Executing UrlPreviewCommand");
        let re = regex::Regex::new(r"(https?://(?:www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}\.[a-z]{2,6}\b(?:[-a-zA-Z0-9@:;%()\[\]{}_\+.*~#?,&//=]*))").unwrap();

        // COMMAND HANDLING
        for cap in re.captures_iter(&msg.text) {
            let url = &cap[1];
            debug!("Previewing url {}", url);
            match reqwest::get(url) {
                Ok(mut resp) => {
                    let mut buf = String::new();
                    resp.read_to_string(&mut buf).expect(
                        "Failed to read response",
                    );
                    let document = Document::from(buf.as_ref());
                    for node in document.find(Name("title")) {
                        println!("{:#?}", node);
                        node.children().for_each(|x| {
                            // SEND MESSAGES
                            let title_irc = x.as_text().unwrap();
                            debug!("extracted url: {}", title_irc);
                            let title_telegram = title_irc;
                            to_irc.send(Message::new(
                                TransportType::Telegram,
                                title_irc.to_owned(),
                                String::from("UrlPreviewCommand"),
                                self.name.to_owned(),
                                true,
                            ));
                            to_telegram.send(Message::new(
                                TransportType::IRC,
                                title_telegram.to_owned(),
                                String::from("UrlPreviewCommand"),
                                self.name.to_owned(),
                                true,
                            ));
                        });
                    }
                },
                Err(e) => {
                    error!("Error previewing: {}", e);
                }
            };
        }
    }
}
