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
    pub is_from_command: bool,
}

impl Message {
    pub fn new(
        from_transport: TransportType,
        text: String,
        from: String,
        to: String,
        is_from_command: bool,
    ) -> Message {
        Message {
            from_transport,
            text,
            from,
            to,
            is_from_command,
        }
    }
    // TODO: sanitize this senseless abuse
    // TODO: handle symbol command for command name
    pub fn strip_command(&self, command_prefix: &str) -> String {
        let re = Regex::new(format!(r"({})\w+\s", command_prefix).as_ref()).unwrap();
        format!("{}: {}", self.from, re.replace_all(&self.text, ""))
    }
}
