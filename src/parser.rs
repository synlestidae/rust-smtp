use std::io::Read;

use parse_util::*;
use data::{Command, ParseError};
use address::Address;

pub fn read_command(stream: &mut Read) -> Result<Command, ParseError> {
    let line = try!(read_line(stream)).into_bytes();
    parse_command(&line)
}

pub fn parse_command(command: &[u8]) -> Result<Command, ParseError> {
    let total_len = command.len();

    if total_len < 2 || command[total_len - 2] != CR || command[total_len - 1] != LF {
        println!("This command wrong: {:?}",
                 String::from_utf8(command.iter().map(|&b| b).collect::<Vec<u8>>()).unwrap());
        return Err(ParseError::InvalidLineEnding);
    }

    let input_line = &command[0..total_len];
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
                line.pop_while(|b: u8| b == (' ' as u8));
                let addr = if line.match_next_str_ignore_case("<") {
                    line.pop_while(|b| b == ' ' as u8);
                    let addr = line.pop_while(|b: u8| b != '>' as u8);
                    if line.is_at_end() {
                        return Err(ParseError::SyntaxError("Invalid MAIL command: \
                                                                    Missing >"));
                    }
                    line.match_next_str_ignore_case(">");
                    addr
                } else {
                    let addr = line.pop_while(|b: u8| b != CR && b != LF && b != (' ' as u8));
                    addr
                };
                if line.match_next_str_ignore_case("\r\n") && line.is_at_end() {
                    let validating_address = _parse_address(addr);
                    if !validating_address.is_ok() {
                        return Err(ParseError::SyntaxError("Address is not a valid email address"));
                    }
                    return Ok(Command::MAIL_FROM(validating_address.unwrap()));
                } else {
                    return Err(ParseError::SyntaxError("Invalid trailing characters on MAIL \
                                                        command"));
                }
            } else {
                return Err(ParseError::SyntaxError("Invalid MAIL command"));
            }
        }
        Some('h') => {
            if line.match_next_bytes_ignore_case(b"ELO ") {
                return Ok(Command::HELO(line.read_line().unwrap()));
            } else {
                return Err(ParseError::MalformedCommand("Expected HELO"));
            }
        }
        Some('e') => {
            if line.match_next_bytes_ignore_case(b"HLO ") {
                return Ok(Command::EHLO(line.read_line().unwrap()));
            } else {
                return Err(ParseError::MalformedCommand("Expected EHLO"));
            }
        }
        Some('r') => {
            if line.match_next_bytes_ignore_case(b"CPT TO:") {
                let email_address = try!(_read_address(&mut line));
                return Ok(Command::RCPT_TO(email_address));
            } else {
                return Err(ParseError::MalformedCommand("Expected RCPT TO"));
            }
        }
        Some('d') => {
            if line.match_next_bytes_ignore_case(b"ATA\r\n") {
                return Ok(Command::DATA);
            } else {
                return Err(ParseError::MalformedCommand("Expected DATA"));
            }
        }
        Some('q') => {
            if line.match_next_bytes_ignore_case(b"UIT\r\n") {
                return Ok(Command::QUIT);
            } else {
                return Err(ParseError::MalformedCommand("Expected QUIT"));
            }
        }
        _ => return Err(ParseError::MalformedCommand("Unknown command")),
    }
}

fn _read_address(line: &mut SliceScanner) -> Result<Address, ParseError> {
    line.pop_while(|b: u8| b == (' ' as u8));
    let addr = if line.match_next_str_ignore_case("<") {
        line.pop_while(|b| b == ' ' as u8);
        let addr = line.pop_while(|b: u8| b != '>' as u8);
        if line.is_at_end() {
            return Err(ParseError::SyntaxError("Invalid MAIL command: Missing >"));
        }
        line.match_next_str_ignore_case(">");
        addr
    } else {
        let addr = line.pop_while(|b: u8| b != CR && b != LF && b != (' ' as u8));
        addr
    };
    if line.match_next_str_ignore_case("\r\n") && line.is_at_end() {
        let validating_address = _parse_address(addr);
        if !validating_address.is_ok() {
            return Err(ParseError::SyntaxError("Address is not a valid email address"));
        }
        return Ok(validating_address.unwrap());
    } else {
        return Err(ParseError::SyntaxError("Invalid trailing characters on MAIL command"));
    }
}

fn _parse_address(addr: Vec<u8>) -> Result<Address, ParseError> {
    let address_string_result = String::from_utf8(addr);
    match address_string_result {
        Err(_) => Err(ParseError::SyntaxError("Address is not valid ASCII string")),
        Ok(address_string) => {
            let address_parts = address_string.split("@").collect::<Vec<_>>();
            if address_parts.len() == 2 {
                Ok(Address::new(&address_parts[0], &address_parts[1]))
            } else {
                Err(ParseError::SyntaxError("Address has invalid number of '@' characters"))
            }
        }
    }
}

#[test]
fn test_commands() {
    fn test_parse_command(input: &str, expected: Result<Command, ParseError>) {
        assert_eq!(expected, parse_command(&input.to_string().into_bytes()));
    }

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
                       Ok(Command::MAIL_FROM(Address::new("mneumann", "ntecs.de"))));
    test_parse_command("MAIL FROM:mneumann@ntecs.de\r\n",
                       Ok(Command::MAIL_FROM(Address::new("mneumann", "ntecs.de"))));


    test_parse_command("DATA\r\n", Ok(Command::DATA));
    test_parse_command("data\r\n", Ok(Command::DATA));
    test_parse_command("data test\r\n",
                       Err(ParseError::MalformedCommand("Expected DATA")));
}
