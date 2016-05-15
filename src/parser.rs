use std::cmp::min;
use std;
use std::io::Read;

use parse_util::*;
use data::{Command, ParseError};

pub fn read_command(stream: &mut Read) -> Result<Command, ParseError> {
    let line = try!(read_line(stream)).into_bytes();
    parse_command(&line)
}

pub fn parse_command(command: &[u8]) -> Result<Command, ParseError> {
    let total_len = command.len();

    if total_len < 2 || command[total_len - 2] != CR && command[total_len - 1] != LF {
        return Err(ParseError::InvalidLineEnding);
    }

    let mut input_line = &command[0..total_len - 2];
    let mut line = SliceScanner::new(input_line);

    match line.pop().map(|b: u8| ignore_ascii_case(b) as char) {
        Some('m') => {
            if !line.match_next_str_ignore_case("AIL") {
                return Err(ParseError::SyntaxError("Malformed commands. Only MAIL FROM begins \
                                                    with M"));
            }

            if line.pop_while(is_space_byte).len() == 0 {
                return Err(ParseError::SyntaxError("Invalid MAIL command: Missing space after \
                                                    MAIL"));
            }

            if line.match_next_str_ignore_case("FROM:") {
                let addr = if line.match_next_str_ignore_case("<") {
                    let _ = line.pop();
                    let addr = line.pop_while(|b: u8| b != '>' as u8 && b != CR && b != LF);
                    println!("Addr: {:?}. At end? {}",
                             String::from_utf8(addr.clone()).unwrap(),
                             line.is_at_end());
                    if !line.is_at_end() {
                        match line.pop() {
                            Some(62) => {} //ASCII for '>'
                            _ => {
                                return Err(ParseError::SyntaxError("Invalid MAIL command: \
                                                                    Missing >"))
                            }
                        }
                    } else {
                        return Err(ParseError::InvalidLineEnding);
                    }
                    addr
                } else {
                    line.pop_while(|b| b != ' ' as u8)
                };

                if line.match_next_str_ignore_case("\r\n") && line.is_at_end() {
                    // XXX: Verify mail addr
                    Ok(Command::MAIL_FROM(String::from_utf8(addr).unwrap()))
                } else {
                    println!("There were trailing characters on mail command: `{}`",
                             std::str::from_utf8(line.data()).unwrap());
                    Err(ParseError::SyntaxError("Invalid trailing characters on MAIL command"))
                }
            } else {
                Err(ParseError::SyntaxError("Invalid MAIL command"))
            }
        }
        Some('h') => {
            if line.match_next_bytes_ignore_case(b"ELO ") {
                Ok(Command::HELO(line.read_line().unwrap()))
            } else {
                Err(ParseError::MalformedCommand)
            }
        }
        Some('e') => {
            if line.match_next_bytes_ignore_case(b"HLO ") {
                Ok(Command::EHLO(line.read_line().unwrap()))
            } else {
                Err(ParseError::MalformedCommand)
            }
        }
        Some('r') => {
            if line.match_next_bytes_ignore_case(b"CPT TO:") {
                Ok(Command::RCPT_TO(line.read_line().unwrap()))
            } else {
                Err(ParseError::MalformedCommand)
            }
        }
        Some('d') => {
            if line.match_next_bytes_ignore_case(b"ATA\r\n") {
                Ok(Command::DATA)
            } else {
                Err(ParseError::MalformedCommand)
            }
        }
        Some('q') => {
            if line.match_next_bytes_ignore_case(b"UIT\r\n") {
                Ok(Command::QUIT)
            } else {
                Err(ParseError::MalformedCommand)
            }
        }
        _ => Err(ParseError::MalformedCommand),
    }
}

fn test_parse_command(input: &str, expected: Result<Command, ParseError>) {
    assert_eq!(expected, parse_command(&input.to_string().into_bytes()));
}

#[test]
fn test_commands() {
    // test_parse_command!("", Err(InvalidLineEnding));
    test_parse_command("\r", Err(ParseError::InvalidLineEnding));
    test_parse_command("\n", Err(ParseError::InvalidLineEnding));
    test_parse_command("\n\r", Err(ParseError::InvalidLineEnding));
    test_parse_command("MAIL FROM:<mneumann@ntecs.de>",
                       Err(ParseError::InvalidLineEnding));
    test_parse_command("MAIL FROM:<mneumann@ntecs.de>\r",
                       Err(ParseError::InvalidLineEnding));
    test_parse_command("MAIL FROM:<mneumann@ntecs.de>\n",
                       Err(ParseError::InvalidLineEnding));
    test_parse_command("MAIL FROM:<mneumann@ntecs.de>\n\r",
                       Err(ParseError::InvalidLineEnding));

    test_parse_command("MAIL FROM:<mneumann@ntecs.de blah\r\n",
                       Err(ParseError::SyntaxError("Invalid MAIL command: Missing >")));

    test_parse_command("MAIL FROM:<mneumann@ntecs.de>\r\n",
                       Ok(Command::MAIL_FROM(String::from("mneumann@ntecs.de"))));
    test_parse_command("MAIL FROM:mneumann@ntecs.de\r\n",
                       Ok(Command::MAIL_FROM(String::from("mneumann@ntecs.de"))));


    test_parse_command("DATA\r\n", Ok(Command::DATA));
    test_parse_command("data\r\n", Ok(Command::DATA));
    test_parse_command("data test\r\n",
                       Err(ParseError::SyntaxError("Invalid DATA command")));
}

fn main() {
    let buf = b"MAIL FROM:<mneumann@ntecs.de>\r\n";
    let cmd = parse_command(buf);
    println!("{:?}", cmd);
}
