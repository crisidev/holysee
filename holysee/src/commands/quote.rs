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
    command_prefix: &'a str,
    data_dir: &'a str,
}

impl<'a> QuoteCommand<'a> {
    pub fn new(command_prefix: &'a str, data_dir: &'a str) -> QuoteCommand<'a> {
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

    fn write_database(&self) -> bool {
        let filename = format!("{}/{}.json", self.data_dir, &self.get_name());
        let filename_clone = filename.clone();
        match OpenOptions::new().write(true).truncate(true).open(filename) {
            Ok(file) => {
                if let Err(e) = serde_json::to_writer(file, &self.quotes) {
                    error!("Cannot serialize file {}: {}", filename_clone, e);
                    return false;
                };
            }
            Err(e) => {
                error!("Cannot open file {}: {}", filename_clone, e);
                return false;
            }
        };
        true
    }

    fn get(&self) -> String {
        if self.quotes.is_empty() {
            return String::from("no quotes in the database");
        }
        let mut rng = rand::thread_rng();
        let range = Range::new(0, self.quotes.len());
        let n: usize = range.ind_sample(&mut rng) as usize;
        self.get_id(n)
    }

    fn get_id(&self, index: usize) -> String {
        if index >= self.quotes.len() as usize {
            return format!("quote #{} does not exist", index);
        }
        format!(
            "quote #{} \"{} - {}\"",
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
            return format!("quote #{} \"{} - {}\" already added", index, quote, author);
        }
        self.quotes.push(Quote::new(
            author,
            quote,
            Local::now().timestamp(),
        ));
        self.write_database();
        format!(
            "quote #{} \"{} - {}\" added",
            self.index(quote),
            quote,
            author
        )
    }

    fn rm(&mut self, quote: &str) -> String {
        let index = self.index(quote);
        if index == -1 {
            return format!("quote \"{}\" does not exist", quote);
        }
        self.rm_id(index as usize)
    }

    fn rm_id(&mut self, index: usize) -> String {
        if index >= self.quotes.len() {
            return format!("quote #{} does not exist", index);
        }
        let quote = self.get_id(index);
        self.quotes.remove(index);
        self.write_database();
        format!("{} removed", quote)
    }

    fn handle(&mut self, text: &str, from: &str) -> String {
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
        let mut result = format!("command \"{}\" not recognized", text);

        // COMMAND HANDLING
        for cap in re_get.captures_iter(text) {
            debug!("Quote get captures {:#?}", cap);
            result = self.get();
        }
        for cap in re_get_id.captures_iter(text) {
            debug!("Quote get id captures {:#?}", cap);
            result = self.get_id(usize::from_str(&cap[1]).unwrap());
        }
        for cap in re_add.captures_iter(text) {
            debug!("Quote add captures {:#?}", cap);
            result = self.add(&cap[1], from);
        }
        for cap in re_rm.captures_iter(text) {
            debug!("Quote rm captures {:#?}", cap);
            result = match cap[1].parse::<usize>() {
                Ok(n) => self.rm_id(n),
                Err(_) => self.rm(&cap[1]),
            }
        }
        result
    }
}

impl<'a> Command for QuoteCommand<'a> {
    fn execute(
        &mut self,
        msg: &mut Message,
        to_irc: &Sender<Message>,
        to_telegram: &Sender<Message>,
    ) {
        let quote_irc = self.handle(&msg.text, &msg.from);
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
    extern crate tempdir;

    use self::tempdir::TempDir;

    use super::{Command, QuoteCommand, Message, TransportType, DestinationType};

    #[test]
    fn test_read_database() {
        assert!(QuoteCommand::read_database("adir", "quote.json").is_err());
        let data_dir = TempDir::new("holysee_quote").unwrap();
        assert!(
            QuoteCommand::read_database(data_dir.path().to_str().unwrap(), "quote.json").is_err()
        );
    }

    #[test]
    fn test_write_database() {
        // TODO: handle also successful case which now returns
        // Cannot open file /var/folders/xj/8kykppps3b9d79m8g40nbyz9p52bt_/T/holysee_quote.C6rvI2j6eqIa/quote.json: No such file or directory (os error 2)
        // Cannot open file /var/folders/xj/8kykppps3b9d79m8g40nbyz9p52bt_/T/holysee_quote.C6rvI2j6eqIa/quote.json: No such file or directory (os error 2)
        let prefix = String::from("!");
        let quote = QuoteCommand::new(&prefix, "adir");
        assert!(!quote.write_database());
    }

    #[test]
    fn test_matches_message_text() {
        let prefix = String::from("!");
        let data_dir = String::from("adir");
        let quote = QuoteCommand::new(&prefix, &data_dir);
        let mut msg = Message {
            from_transport: TransportType::IRC,
            text: String::from("!quote"),
            from: String::from("auser"),
            to: DestinationType::User(String::from("auser")),
            is_from_command: false,
        };

        let success = [
            "!quote",
            "!quote add aquote",
            "!quote rm aquote",
            "!quote 3",
            "!Quote",
            "!Quote add aquote",
            "!Quote rm aquote",
            "!Quote 3",
        ];
        for text in success.iter() {
            msg.text = String::from(*text);
            assert!(quote.matches_message_text(&msg));
        }

        let failures = [
            "quote",
            "quote add aquote",
            "quote rm aquote",
            "quote 3",
            "quote ",
            "Quote",
            "Quote add aquote",
            "Quote rm aquote",
            "Quote 3",
            "Quote ",
            "astring",
        ];
        for text in failures.iter() {
            msg.text = String::from(*text);
            assert!(!quote.matches_message_text(&msg));
        }
    }

    #[test]
    fn test_handle() {
        let prefix = String::from("!");
        let data_dir = TempDir::new("holysee_quote").unwrap();
        let mut quote = QuoteCommand::new(&prefix, data_dir.path().to_str().unwrap());

        let cases = [
            ["!quote", "no quotes in the database"],
            ["!quote add aquote", "quote #0 \"aquote - auser\" added"],
            [
                "!quote add aquote",
                "quote #0 \"aquote - auser\" already added",
            ],
            ["!quote", "quote #0 \"aquote - auser\""],
            ["!quote 0", "quote #0 \"aquote - auser\""],
            ["!quote 1", "quote #1 does not exist"],
            ["!quote rm aquote", "quote #0 \"aquote - auser\" removed"],
            ["!quote rm aquote", "quote \"aquote\" does not exist"],
            ["!quote rm 0", "quote #0 does not exist"],
            ["quote", "command \"quote\" not recognized"],
            ["Quote", "command \"Quote\" not recognized"],
            ["astring", "command \"astring\" not recognized"],
        ];
        for case in cases.iter() {
            assert!(quote.handle(case[0], "auser") == *case[1]);
        }
    }
}
