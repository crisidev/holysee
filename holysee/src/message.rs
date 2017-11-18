extern crate regex;

use self::regex::Regex;

#[derive(Debug)]
pub enum TransportType {
    IRC,
    Telegram,
}

#[derive(Debug)]
pub struct Message {
    pub from_transport: TransportType,
    pub text: String,
    pub from: String,
    pub to: String,
}

impl Message {
    pub fn format(&self) -> String {
        // remove command if present
        let re = Regex::new(r"!\w+\s").unwrap();
        format!("{}: {}", self.from, re.replace_all(&self.text, ""))
    }
}
