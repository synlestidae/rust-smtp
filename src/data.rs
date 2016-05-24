use address::Address;

#[derive(PartialEq, Eq, Debug)]
#[allow(non_camel_case_types)]
pub enum Command {
    HELO(String),
    EHLO(String),
    MAIL_FROM(Address),
    RCPT_TO(Address),
    DATA,
    QUIT,
    VERIFY,
    RESET,
    NOOP
}

#[allow(dead_code)]
#[derive(Eq, PartialEq, Debug)]
pub enum ParseError {
    SyntaxError(&'static str),
    InvalidLineEnding,
    UnknownCommand,
    MalformedCommand(&'static str),
    UnexpectedEndOfInput,
}
