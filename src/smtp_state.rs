use data::Command;
use response::Response;

#[derive(Copy, Clone)]
pub enum SmtpState {
    Start,
    ReadyForRecptTo,
    ReadyForData,
    DataInProgress,
    ReadyToProcess,
    Quit,
}

pub enum SmtpError {
    UnknownCommand,
}

pub const OK: u16 = 250;

pub trait SmtpStateMachine {
    fn state(&self) -> SmtpState;
    fn transition(&mut self, cmd: &Command) -> Result<Response, SmtpError>;
}

pub struct DefaultStateMachine {
    state: SmtpState,
}

impl DefaultStateMachine {
    pub fn new() -> DefaultStateMachine {
        DefaultStateMachine { state: SmtpState::Start }
    }
}

impl SmtpStateMachine for DefaultStateMachine {
    fn state(&self) -> SmtpState {
        self.state
    }

    fn transition(&mut self, cmd: &Command) -> Result<Response, SmtpError> {
        match (self.state, cmd) {
            (SmtpState::Start, &Command::MAIL_FROM(ref mailfrom)) => {
                self.state = SmtpState::ReadyForRecptTo;
                Ok(Response::new(OK, "OK"))
            }
            (SmtpState::ReadyForRecptTo, &Command::RCPT_TO(ref mailto)) => {
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
            _ => Err(SmtpError::UnknownCommand),
        }
    }
}
