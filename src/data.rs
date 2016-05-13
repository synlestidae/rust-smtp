#[derive(PartialEq, Eq, Debug)]
#[allow(non_camel_case_types)]
pub enum Command {
    HELO(String),
    EHLO(String),
    MAIL_FROM(String),
    RCPT_TO(String),
    DATA,
    QUIT,
    Invalid
}

#[allow(dead_code)]
#[derive(Eq, PartialEq, Debug)]
pub enum ParseError {
    SyntaxError(&'static str),
    InvalidLineEnding,
    UnknownCommand,
    UnexpectedEndOfInput
}