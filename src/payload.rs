use address::Address;

pub struct Payload {
    pub recipients: Vec<Address>,
    pub data: Vec<u8>,
}

impl Payload {
    pub fn add_recipient(&mut self, recipient: Address) {
        self.recipients.push(recipient);
    }
}
