use std::io::Error;
use data::{Command, ParseError};

#[derive(Debug)]
pub enum SmtpError {
    UnexpectedCommand(Command, &'static str),
    IOError(Error),
    ParseError(ParseError),
}

pub enum ErrorLevel {
    Fatal,
    Retryable,
    NA,
}
