use payload::Payload;
use email::MimeMessage;
use lettre::email::{EmailBuilder, SendableEmail};
use lettre::transport::smtp::{SecurityLevel, SmtpTransport,
SmtpTransportBuilder};
use lettre::transport::smtp::authentication::Mechanism;
use lettre::transport::smtp::SUBMISSION_PORT;
use lettre::transport::EmailTransport;
use rand::Rng;
use rand;


pub struct PayloadHandler;

struct PayloadEmail(Payload, MessageId);

type MessageId = String;

impl SendableEmail for PayloadEmail {
    fn from_address(&self) -> String {
        match self.0.sender {
            Some(ref sender) => format!("{}@{}", sender.address, sender.domain),
            _ => panic!("No address set on payload!")
        }
    }
    fn to_addresses(&self) -> Vec<String> {
        self.0.recipients.iter().map(|rec| format!("{}@{}", rec.address, rec.domain)).collect::<Vec<String>>()
    }
    fn message(&self) -> String{
        String::from_utf8(self.0.data.clone()).unwrap()
    }
    fn message_id(&self) -> String{
        self.1.clone()
    }
}

impl PayloadHandler {
	pub fn handle(payload: Payload) {
        let data_string = String::from_utf8(payload.data.clone()).unwrap();
        let message = MimeMessage::parse(&data_string).unwrap();
        let message_id: String = match message.headers.get("Message-ID".to_string()) {
            Some(msg_id) => msg_id.get_value::<String>().unwrap().clone(),
            _ => {
                let mut rng = rand::thread_rng();
                let mut available = Vec::new();
                available.extend('A' as u8..'Z' as u8);
                available.extend('a' as u8..'z' as u8);
                available.extend('0' as u8..'9' as u8);
                String::from_utf8((0..48).map(|x| available[rng.gen::<usize>() % available.len()])
                    .collect::<Vec<u8>>()).ok().unwrap()
            }
        };
        let payload_email = PayloadEmail(payload, message_id);
        let mut mailer = SmtpTransportBuilder::new(("server.tld",
                                       SUBMISSION_PORT)).unwrap()
                // Set the name sent during EHLO/HELO, default is `localhost`
                .hello_name("210-246-36-65.dsl.dyn.ihug.co.nz")
                    // Add credentials for authentication
                    .credentials("just.mate.antunovic@gmail.com", "shittycrap12")
                        // Specify a TLS security level. You can also specify an SslContext with
                        // .ssl_context(SslContext::Ssl23)
                        .security_level(SecurityLevel::AlwaysEncrypt)
                            // Enable SMTPUTF8 if the server supports it
                            .smtp_utf8(true)
                                // Configure expected authentication mechanism
                                //.authentication_mechanism(Mechanism::CramMd5)
                                    // Enable connection reuse
                                    .connection_reuse(false).build();
        mailer.send(payload_email);
	}
}
