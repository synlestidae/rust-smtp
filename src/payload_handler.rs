use payload::Payload;
use email::MimeMessage;
use lettre::email::SendableEmail;

pub struct PayloadHandler;

pub struct PayloadEmail(Payload);

impl SendableEmail for Payload {
    fn from_address(&self) -> String {
        panic!("implemente me");
    }
    fn to_addresses(&self) -> Vec<String> {
        panic!("implemente me");
    }
    fn message(&self) -> String{
        panic!("implemente me");
    }
    fn message_id(&self) -> String{
        panic!("implemente me");
    }
}

impl PayloadHandler {
	pub fn handle(payload: Payload) {
        let data_string = String::from_utf8(payload.data).unwrap();
        let message = MimeMessage::parse(&data_string).unwrap();
	}
}
