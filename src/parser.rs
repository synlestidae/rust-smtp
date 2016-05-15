use std::cmp::min;
use std;
use std::io::Read;

use parse_util::*;
use data::{Command, ParseError};

pub fn read_command(stream: &Read) -> Result<Command, ParseError> {
    panic!("Not implemented")
}

pub fn parse_command(command: &[u8]) -> Result<Command, ParseError> {
    let total_len = command.len();

    if total_len < 2 || command[total_len - 2] != CR && command[total_len - 1] != LF {
        return Err(ParseError::InvalidLineEnding);
    }

    let mut input_line = &command[0..total_len - 2];
    let mut line = SliceScanner::new(input_line);

    match line.pop().map(|b: u8| b as char) {
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
                    let addr = line.pop_while(is_not_less_than);
                    if line.pop() != Some('>' as u8) {
                        return Err(ParseError::SyntaxError("Invalid MAIL command: Missing >"));
                    }
                    addr
                } else {
                    line.pop_while(is_not_space_byte)
                };

                line.match_next_str_ignore_case("\r\n");

                if line.is_at_end() {
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
                Ok(Command::Invalid)
            }
        }
        Some('e') => {
            if line.match_next_bytes_ignore_case(b"HLO ") {
                Ok(Command::EHLO(line.read_line().unwrap()))
            } else {
                Ok(Command::Invalid)
            }
        }
        Some('r') => {
            if line.match_next_bytes_ignore_case(b"CPT TO:") {
                Ok(Command::RCPT_TO(line.read_line().unwrap()))
            } else {
                Ok(Command::Invalid)
            }
        }
        Some('d') => {
            if line.match_next_bytes_ignore_case(b"ATA\r\n") {
                Ok(Command::DATA)
            } else {
                Ok(Command::Invalid)
            }
        }
        Some('q') => {
            if line.match_next_bytes_ignore_case(b"UIT\r\n") {
                Ok(Command::QUIT)
            } else {
                Ok(Command::Invalid)
            }
        }
        _ => Ok(Command::Invalid),
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
                       Ok(Command::MAIL("mneumann@ntecs.de")));
    test_parse_command("MAIL FROM:mneumann@ntecs.de\r\n",
                       Ok(Command::MAIL("mneumann@ntecs.de")));


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
