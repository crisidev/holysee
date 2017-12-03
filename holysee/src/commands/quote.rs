extern crate regex;
extern crate serde_json;
extern crate chrono;
extern crate rand;

use chan::Sender;
use std::fs::OpenOptions;
use std::error::Error;
use std::boxed::Box;
use std::str::FromStr;

use self::regex::Regex;
use self::chrono::Local;
use self::rand::distributions::{IndependentSample, Range};

use message::{Message, TransportType, DestinationType};
use commands::command_dispatcher::Command;

#[derive(Debug, Serialize, Deserialize)]
struct Quote {
    pub author: String,
    pub quote: String,
    pub date: i64,
}

impl Quote {
    fn new(author: &str, quote: &str, date: i64) -> Quote {
        Quote {
            author: author.to_owned(),
            quote: quote.to_owned(),
            date,
        }
    }
}

#[derive(Debug)]
pub struct QuoteCommand<'a> {
    quotes: Vec<Quote>,
    command_prefix: &'a String,
    data_dir: &'a String,
}

impl<'a> QuoteCommand<'a> {
    pub fn new(command_prefix: &'a String, data_dir: &'a String) -> QuoteCommand<'a> {
        QuoteCommand {
            quotes: match QuoteCommand::read_database(data_dir, "quote") {
                Ok(v) => v,
                Err(b) => {
                    error!("error reading database: {}", b);
                    vec![]
                }
            },
            command_prefix,
            data_dir: data_dir,
        }
    }


    fn read_database(data_dir: &str, name: &str) -> Result<Vec<Quote>, Box<Error>> {
        let filename = format!("{}/{}.json", data_dir, &name);
        let filename_clone = filename.clone();
        let file = OpenOptions::new().read(true).open(filename)?;
        serde_json::from_reader(file).or_else(|e| {
            Err(From::from(
                format!("Cannot deserialize file {}: {}", filename_clone, e),
            ))
        })
    }

    fn write_database(&self) {
        let filename = format!("{}/{}.json", self.data_dir, &self.get_name());
        let filename_clone = filename.clone();
        match OpenOptions::new().write(true).truncate(true).open(filename) {
            Ok(file) => {
                if let Err(e) = serde_json::to_writer(file, &self.quotes) {
                    error!("Cannot serialize file {}: {}", filename_clone, e)
                };
            }
            Err(e) => error!("Cannot open file {}: {}", filename_clone, e),
        };
    }

    fn get(&self) -> String {
        if self.quotes.is_empty() {
            return String::from("No quotes in the database");
        }
        let mut rng = rand::thread_rng();
        let range = Range::new(0, self.quotes.len());
        let n: usize = range.ind_sample(&mut rng) as usize;
        self.get_id(n)
    }

    fn get_id(&self, index: usize) -> String {
        if index >= self.quotes.len() as usize {
            return format!("no quote with id {} found", index);
        }
        format!(
            "quote #{} \"{}\" - {}",
            index,
            self.quotes[index].quote,
            self.quotes[index].author
        )
    }

    fn index(&self, quote: &str) -> i64 {
        match self.quotes.iter().position(|x| x.quote == quote) {
            Some(n) => n as i64,
            None => -1,
        }
    }

    fn add(&mut self, quote: &str, author: &str) -> String {
        let index = self.index(quote);
        if index != -1 {
            return format!("quote #{} \"{}\" already added", index, quote);
        }
        self.quotes.push(Quote::new(
            author,
            quote,
            Local::now().timestamp(),
        ));
        self.write_database();
        format!("quote #{} \"{}\" added", self.index(quote), quote)
    }

    fn rm(&mut self, quote: &str) -> String {
        let index = self.index(quote);
        if index == -1 {
            return format!("quote \"{}\" does not exist", quote);
        }
        self.rm_id(index as usize)
    }

    fn rm_id(&mut self, index: usize) -> String {
        if index > self.quotes.len() {
            return format!("quote #{} does not exist", index);
        }
        let quote = self.get_id(index);
        self.quotes.remove(index);
        self.write_database();
        format!("quote #{} \"{}\" removed", index, quote)
    }
}

impl<'a> Command for QuoteCommand<'a> {
    fn execute(
        &mut self,
        msg: &mut Message,
        to_irc: &Sender<Message>,
        to_telegram: &Sender<Message>,
    ) {
        let re_get = Regex::new(
            format!(r"^(?:{})[qQ]uote(?:\s+)?$", self.command_prefix).as_ref(),
        ).unwrap();
        let re_get_id = Regex::new(
            format!(r"^(?:{})[qQ]uote(?:\s+)(\d+)$", self.command_prefix).as_ref(),
        ).unwrap();
        let re_add = Regex::new(
            format!(
                r"^(?:{})[qQ]uote(?:\s+)add(?:\s+)(.*)$",
                self.command_prefix
            ).as_ref(),
        ).unwrap();
        let re_rm = Regex::new(
            format!(r"^(?:{})[qQ]uote(?:\s+)rm(?:\s+)(.*)$", self.command_prefix).as_ref(),
        ).unwrap();
        let mut quote_irc = format!("command \"{}\" not recognized", msg.text);

        // COMMAND HANDLING
        for cap in re_get.captures_iter(&msg.text) {
            debug!("Quote get captures {:#?}", cap);
            quote_irc = self.get();
        }
        for cap in re_get_id.captures_iter(&msg.text) {
            debug!("Quote get id captures {:#?}", cap);
            quote_irc = self.get_id(usize::from_str(&cap[1]).unwrap());
        }
        for cap in re_add.captures_iter(&msg.text) {
            debug!("Quote add captures {:#?}", cap);
            quote_irc = self.add(&cap[1], &msg.from);
        }
        for cap in re_rm.captures_iter(&msg.text) {
            debug!("Quote rm captures {:#?}", cap);
            quote_irc = match cap[1].parse::<usize>() {
                Ok(n) => self.rm_id(n),
                Err(_) => self.rm(&cap[1]),
            }
        }

        let quote_telegram = quote_irc.clone();
        let destination = match msg.to {
            DestinationType::Channel(ref c) => DestinationType::Channel(c.clone()),
            DestinationType::User(_) => DestinationType::User(msg.from.clone()),
            DestinationType::Unknown => panic!("Serious bug in quote command handler"),
        };
        let destination_irc: DestinationType = DestinationType::klone(&destination);
        let destination_telegram: DestinationType = DestinationType::klone(&destination);

        // SEND MESSAGES
        match msg.from_transport {
            TransportType::IRC => {
                to_irc.send(Message::new(
                    TransportType::Telegram,
                    quote_irc,
                    String::from("QuoteCommand"),
                    destination_irc,
                    true,
                ));
            }
            TransportType::Telegram => {
                to_telegram.send(Message::new(
                    TransportType::IRC,
                    quote_telegram,
                    String::from("QuoteCommand"),
                    destination_telegram,
                    true,
                ));
            }
        }
    }

    fn get_usage(&self) -> String {
        String::from(
            "\
The quote command maintains a list of quotes. To get a random quote run\
    !quote\
to add a quote use\
    !quote add <quote>\
to delete a quote use\
    !quote rm <quote_id>\
to get a specific quote run\
    !quote <quote_id>",
        )
    }

    fn get_name(&self) -> String {
        String::from("quote")
    }

    fn matches_message_text(&self, message: &Message) -> bool {
        let re = Regex::new(
            // the shame cannot be forgotten
            format!(r"^(?:{})(?:[qQ]uote)(.*)$", self.command_prefix).as_ref(),
        ).unwrap();
        re.is_match(&message.text)
    }

    fn stop_processing(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, QuoteCommand, Message, TransportType, DestinationType};

    #[test]
    fn test_matches_message_text_ok() {
        let prefix = String::from("!");
        let data_dir = String::from("adir");
        let quote = QuoteCommand::new(&prefix, &data_dir);
        let mut msg = Message{
            from_transport: TransportType::IRC,
            text: String::from("!quote"),
            from: String::from("auser"),
            to: DestinationType::User(String::from("auser")),
            is_from_command: false,
        };
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("!quote add aquote");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("!quote rm aquote");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("!quote 3");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("!Quote");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("!Quote add aquote");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("!Quote rm aquote");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("!Quote 3");
        assert!(quote.matches_message_text(&msg));
    }

    #[test]
    #[should_panic]
    fn test_matches_message_text_ko() {
        let prefix = String::from("!");
        let data_dir = String::from("adir");
        let quote = QuoteCommand::new(&prefix, &data_dir);
        let mut msg = Message{
            from_transport: TransportType::IRC,
            text: String::from("quote"),
            from: String::from("auser"),
            to: DestinationType::User(String::from("auser")),
            is_from_command: false,
        };
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("quote add aquote");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("quote rm aquote");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("quote 3");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("quote ");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("Quote");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("Quote add aquote");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("Quote rm aquote");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("Quote 3");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("Quote ");
        assert!(quote.matches_message_text(&msg));
        msg.text = String::from("astring");
        assert!(quote.matches_message_text(&msg));
    }
}
