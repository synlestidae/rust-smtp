use data::Command;
use response::Response;
use payload::Payload;
use std::mem;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SmtpState {
    Start,
    ReadyForRecptTo,
    ReadyForData,
    DataInProgress,
    ReadyToProcess,
    Quit
}

pub enum SmtpError {
    UnknownCommand,
}

pub const OK: u16 = 250;

pub trait SmtpStateMachine {
    fn new() -> Self;
    fn state(&self) -> SmtpState;
    fn transition(&mut self, cmd: &Command) -> Result<Response, SmtpError>;
    fn extract_payload(&mut self) -> Payload;
    fn get_payload_mut_ref<'a>(&'a mut self) -> &'a mut Payload;
}

pub struct DefaultStateMachine {
    state: SmtpState,
    current_payload: Payload,
}


impl SmtpStateMachine for DefaultStateMachine {
    fn new() -> DefaultStateMachine {
        DefaultStateMachine {
            state: SmtpState::Start,
            current_payload: Payload::new(),
        }
    }

    fn state(&self) -> SmtpState {
        self.state
    }

    fn transition(&mut self, cmd: &Command) -> Result<Response, SmtpError> {
        match (self.state, cmd) {
            (SmtpState::Start, &Command::MAIL_FROM(ref sender)) => {
                self.current_payload.sender = Some(sender.clone());
                self.state = SmtpState::ReadyForRecptTo;
                Ok(Response::new(OK, "OK"))
            }
            (SmtpState::ReadyForRecptTo, &Command::RCPT_TO(ref recipient)) => {
                self.current_payload.recipients.push(recipient.clone());
                self.state = SmtpState::ReadyForData;
                Ok(Response::new(OK, "OK"))
            }
            (SmtpState::ReadyForData, &Command::DATA) => {
                self.state = SmtpState::DataInProgress;
                Ok(Response::new(354, "End data with <CR><LF>.<CR><LF>"))
            }
            (_, &Command::QUIT) => {
                self.state = SmtpState::Quit;
                Ok(Response::new(221, "Bye"))
            }
            (_, &Command::RESET) => {
                self.state = SmtpState::Start;
                self.current_payload = Payload::new();
                Ok(Response::new(OK, "OK"))
            }
            (_, &Command::NOOP) => {
                Ok(Response::new(OK, "OK"))   
            }
            _ => Err(SmtpError::UnknownCommand),
        }
    }

    fn extract_payload(&mut self) -> Payload {
        let mut swapped_payload = Payload::new();
        mem::swap(&mut swapped_payload, &mut self.current_payload);
        swapped_payload
    }

    fn get_payload_mut_ref<'a>(&'a mut self) -> &'a mut Payload {
        &mut self.current_payload
    }
}
