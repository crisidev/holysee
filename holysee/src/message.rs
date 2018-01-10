extern crate regex;

use self::regex::Regex;
use settings::NickEntry;

#[derive(Debug)]
pub enum TransportType {
    IRC,
    Telegram,
}

#[derive(Debug, Clone)]
pub enum DestinationType {
    Channel(String),
    User(String),
    Unknown,
}

impl DestinationType {
    pub fn klone(other: &DestinationType) -> DestinationType {
        match *other {
            DestinationType::Channel(ref s) => DestinationType::Channel(s.clone()),
            DestinationType::User(ref u) => DestinationType::User(u.clone()),
            DestinationType::Unknown => DestinationType::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub from_transport: TransportType,
    pub text: String,
    pub from: String,
    pub to: DestinationType,
    pub is_from_command: bool,
}

impl Message {
    pub fn new(
        from_transport: TransportType,
        text: String,
        from: String,
        to: DestinationType,
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
        let re = Regex::new(format!(r"^({})\w+\s", command_prefix).as_ref()).unwrap();
        if self.is_from_command {
            format!("{}", re.replace_all(&self.text, ""))
        } else {
            format!("{}: {}", self.from, re.replace_all(&self.text, ""))
        }
    }

    fn nickname_needs_conversion(&self, irc_nick: &str, telegram_nick: &str) -> bool {
        let nick = match self.from_transport {
            TransportType::IRC => {
                irc_nick
            },
            TransportType::Telegram => {
                telegram_nick
            }
        };
        let column_mention = format!("{}:", nick);
        let at_mention = format!("@{}", nick);
        let beginning_mention = format!("{} ", nick);
        let ending_mention = format!(" {}", nick);
        let bare_mention = format!(" {} ", nick);
        self.text.contains(&column_mention) ||
            self.text.contains(&at_mention) ||
            self.text.contains(&beginning_mention) ||
            self.text.contains(&ending_mention) ||
            self.text.contains(&bare_mention)
    }

    // TODO: refactor this interface to not depend on settings::NickEntry
    pub fn convert_nicknames(&mut self, nicknames: &[NickEntry]) {
        for nick_map in nicknames {
            match self.from_transport {
                TransportType::IRC => {
                    if self.nickname_needs_conversion( &nick_map.irc, &nick_map.telegram) {
                        debug!(
                            "Converting current irc from {} to telegram {}",
                            self.from,
                            nick_map.telegram
                        );
                        self.text = self.text.replace(&nick_map.irc, &nick_map.telegram);
                    }
                }
                TransportType::Telegram => {
                    if self.nickname_needs_conversion(&nick_map.irc, &nick_map.telegram) {
                        debug!(
                            "Converting current telegram from {} to irc {}",
                            self.from,
                            nick_map.irc
                        );
                        self.text = self.text.replace(&nick_map.telegram, &nick_map.irc);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Message;
    use super::TransportType;
    use super::DestinationType;

    #[test]
    fn test_strip_command() {
        // from command
        assert_eq!(Message::new(TransportType::IRC, String::from("!command at the beginning of line"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), true).strip_command("!"), String::from("at the beginning of line"));
        // not from command, so contains the nickname
        assert_eq!(Message::new(TransportType::IRC, String::from("!command at the beginning of line"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).strip_command("!"), String::from("nickname: at the beginning of line"));
        // ironic use of ! from command
        assert_eq!(Message::new(TransportType::IRC, String::from("at the !beginning of line"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), true).strip_command("!"), String::from("at the !beginning of line"));
        // ironic use of ! not from command, so contains the nickname
        assert_eq!(Message::new(TransportType::IRC, String::from("at the !beginning of line"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).strip_command("!"), String::from("nickname: at the !beginning of line"));
    }

    #[test]
    fn test_nickname_needs_conversion() {
        // TODO fix this madness
        // IRC message cases
        assert_eq!(Message::new(TransportType::IRC, String::from("nickname: some message"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).nickname_needs_conversion("nickname", "@tg_nickname"), true);
        assert_eq!(Message::new(TransportType::IRC, String::from("@nickname some message"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).nickname_needs_conversion("nickname", "@tg_nickname"), true);
        assert_eq!(Message::new(TransportType::IRC, String::from("@nickname: some message"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).nickname_needs_conversion("nickname", "@tg_nickname"), true);
        assert_eq!(Message::new(TransportType::IRC, String::from("http://some.site.com/~nickname/file.html"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).nickname_needs_conversion("nickname", "@tg_nickname"), false);
        assert_eq!(Message::new(TransportType::IRC, String::from("nickname some message"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).nickname_needs_conversion("nickname", "@tg_nickname"), true);
        assert_eq!(Message::new(TransportType::IRC, String::from("mentioned nickname in a conversation"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).nickname_needs_conversion("nickname", "@tg_nickname"), true);
        assert_eq!(Message::new(TransportType::IRC, String::from("mentioned @nickname in a conversation"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).nickname_needs_conversion("nickname", "@tg_nickname"), true);

        // TELEGRAM message cases
        assert_eq!(Message::new(TransportType::Telegram, String::from("tg_nickname: some message"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).nickname_needs_conversion("nickname", "@tg_nickname"), false);
        assert_eq!(Message::new(TransportType::Telegram, String::from("@tg_nickname some message"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).nickname_needs_conversion("nickname", "@tg_nickname"), true);
        assert_eq!(Message::new(TransportType::Telegram, String::from("@tg_nickname: some message"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).nickname_needs_conversion("nickname", "@tg_nickname"), true);
        assert_eq!(Message::new(TransportType::Telegram, String::from("http://some.site.com/~tg_nickname/file.html"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).nickname_needs_conversion("nickname", "@tg_nickname"), false);
        assert_eq!(Message::new(TransportType::Telegram, String::from("tg_nickname some message"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).nickname_needs_conversion("nickname", "@tg_nickname"), false);
        assert_eq!(Message::new(TransportType::Telegram, String::from("mentioned tg_nickname in a conversation"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).nickname_needs_conversion("nickname", "@tg_nickname"), false);
        assert_eq!(Message::new(TransportType::Telegram, String::from("mentioned @tg_nickname in a conversation"), String::from("nickname"), DestinationType::Channel(String::from("#somechan")), false).nickname_needs_conversion("nickname", "@tg_nickname"), true);
    }
}
