#[derive(Debug)]
pub enum TransportType {
    IRC,
    Telegram,
}

#[derive(Debug)]
pub struct Message {
    pub transport: TransportType,
    pub text: String,
    pub from: String,
    pub to: String,
}
