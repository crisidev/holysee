extern crate regex;

use message::{Message, TransportType,DestinationType};
use chan::Sender;

use self::regex::Regex;
use commands::command_dispatcher::Command;

#[derive(Debug)]
pub struct RelayMessageCommand<'a> {
    pub name: String,
    irc_allow_receive: &'a bool,
    telegram_allow_receive: &'a bool,
    command_prefix: &'a String,
}

impl<'a> RelayMessageCommand<'a> {
    pub fn new(
        irc_allow_receive: &'a bool,
        telegram_allow_receive: &'a bool,
        command_prefix: &'a String,
    ) -> RelayMessageCommand<'a> {
        RelayMessageCommand {
            name: String::from("relay"),
            irc_allow_receive,
            telegram_allow_receive,
            command_prefix,
        }
    }
    pub fn matches_message_text(&self, message: &Message) -> bool {
        match message.from_transport {
            TransportType::IRC => {
                if *self.telegram_allow_receive || message.is_from_command {
                    return true;
                }
            }
            TransportType::Telegram => {
                if *self.irc_allow_receive || message.is_from_command {
                    return true;
                }
            }
        };
        let re = Regex::new(
            format!(r"^({})(irc|tg)\s+(.*)$", self.command_prefix).as_ref(),
        ).unwrap();
        re.is_match(&message.text)
    }
}

impl<'a> Command for RelayMessageCommand<'a> {
    fn execute(
        &mut self,
        msg: &Message,
        irc_sender: &Sender<Message>,
        telegram_sender: &Sender<Message>,
    ) {
        let destination_irc: DestinationType = DestinationType::klone(&msg.to);
        let destination_telegram: DestinationType = DestinationType::klone(&msg.to);
        match msg.from_transport {
            TransportType::IRC => {
                debug!("Sending message to Telegram chan");
                telegram_sender.send(Message::new(
                    TransportType::IRC,
                    msg.strip_command(self.command_prefix),
                    msg.from.clone(),
                    destination_telegram,
                    msg.is_from_command,
                ));
            }
            TransportType::Telegram => {
                debug!("Sending message to IRC chan");
                irc_sender.send(Message::new(
                    TransportType::Telegram,
                    msg.strip_command(self.command_prefix),
                    msg.from.clone(),
                    destination_irc,
                    msg.is_from_command,
                ));
            }
        }
    }

    fn get_usage(&self) -> String {
        return String::from("\
The relay command allows to bypass the configuration allow_receive for a given transport\
If the relay is configured with allow_receive set to false then only messages that start with\
    !<ID> ..\
will be relayed. So for example if you have allow_receive set to false for the telegram tranport\
you will need to use\
    !tg <message>\
for message to be delivered to the chat. Similarly, use !irc for IRC.")
    }
}
