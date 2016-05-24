use address::Address;

pub struct Payload {
    pub sender: Option<Address>,
    pub recipients: Vec<Address>,
    pub data: Vec<u8>
}

impl Payload {
    pub fn new() -> Payload {
        Payload {
            sender: None,
            recipients: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn add_recipient(&mut self, recipient: Address) {
        self.recipients.push(recipient);
    }
}
