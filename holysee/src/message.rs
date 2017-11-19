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
    // TODO: sanitize this senseless abuse
    // TODO: handle symbol command for command name
    pub fn strip_command(&self, command_prefix: &String) -> String {
        let re = Regex::new(format!(r"({})\w+\s", command_prefix).as_ref()).unwrap();
        format!("{}", re.replace_all(&self.text, ""))
    }
}
