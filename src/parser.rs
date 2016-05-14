use std::cmp::min;
use std;
use std::io::Read;
use std::cmp::min;

use parse_util::*;
use data::{Command, ParseError};

// RFC 5321 Section 2.3.8. Lines
const CR: u8 = 0x0D;
const LF: u8 = 0x0A;

fn read_line(io: &mut ReadScanner<u8>) -> Result<String, ParseError> {
    let mut s = "".to_string();

    loop {
        match io.pop() {
            CR => break,
            LF => {
                return Err(ParseError::SyntaxError("Expected CR (before LF). Got LF"));
            }
            byte => {
                s.push(byte as char);
            }
        }
    }

    if pop_front_byte_ignore_case(io) == (LF as u8) {
        Ok(s)
    } else {
        Err(ParseError::InvalidLineEnding)
    }
}

struct SliceScanner<'a> {
    data: &'a [u8],
    index: usize
}

impl<'a> SliceScanner<'a> {
    pub fn new(data: &'a [u8]) -> SliceScanner {
        SliceScanner {
            data: data,
            index: 0
        }
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.index < data.len() {
            self.index = self.index + 1;
            Some(data[index - 1])
        }
        else {
            None
        }
    }

    pub fn is_at_end() -> bool {
        data.len() == self.index
    }

    pub fn pop_many(&mut self, many: usize) -> Vec<u8> {
        let out = data[self.index..(self.index + many)]
            .iter()
            .map(|&b| b)
            .collect::<Vec<u8>>();
        self.index = min(self.index + self.data.len(), self.data.len());
        out
    }

    pub fn pop_while(&mut self, predicate: fn(u8) -> bool) -> Vec<u8> {
        let mut result = Vec::new();
        while self.index < self.data.len() {
            match self.data[self.index] {
                b => {
                    if predicate(b) {
                        result.push(b);
                        self.index = self.index + 1;
                    }
                    else {
                        break;
                    }
                }
            }
        }
        result
    }

    pub fn match_next_bytes_ignore_case(&mut self, expected_bytes: &[u8]) -> bool {
        if self.index + chars.len() < self.data.len() {
            return false;
        }

        let expected_len = expected_bytes.len();

        let is_match = &expected_bytes[self.index..(self.index + expected_len)]
            .iter()
            .zip(expected_bytes)
            .filter(|&(&a, &b)| ascii_eq_ignore_case(a, b))
            .collect::<Vec<_>>()
            .len() == expected_len;

        if is_match {
            self.index = self.index + expected_len;
        }

        is_match
    }

    pub fn match_next_str_ignore_case(&mut self, expected_str: &str) -> bool {
        self.match_next_bytes_ignore_case(&expected_str.bytes().collect::<Vec<u8>>())
    }
}

const CR: u8 = 0x0D;
const LF: u8 = 0x0A;

fn read_command(stream: &Read) -> Result<Command, ParseError> {

}

fn parse_command(command: &[u8]) -> Result<Command, ParseError> {
    let total_len = command.len();

    if total_len < 2 || command[total_len - 2] != CR && command[total_len - 1] != LF {
        return Err(ParseError::InvalidLineEnding);
    }

    let mut input_line = &command[0..total_len - 2];
    let mut line = SliceScanner::new(input_line);

    match line.pop().map(|b| b as char) {
        Some('m') => {
            if !line.match_next_str_ignore_case("AIL") {
                return Err(ParseError::SyntaxError("Malformed commands. Only MAIL FROM begins with M"));
            }

            if line.pop_while(is_space_byte).len() == 0 {
                return Err(ParseError::SyntaxError("Invalid MAIL command: Missing space after MAIL"));
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

                if line.is_at_end() {
                    // XXX: Verify mail addr
                    Ok(Command::MAIL_FROM(String::from_utf8(addr).unwrap()))
                } else {
                    println!("There were trailing characters on mail command: `{}`", std::str::from_utf8(line.data).unwrap());
                    Err(ParseError::SyntaxError("Invalid trailing characters on MAIL command"))
                }
            } else {
                Err(ParseError::SyntaxError("Invalid MAIL command"))
            }
        },
        Some('h') => {
            if line.match_next_bytes_ignore_case(b"ELO ") {
                Ok(Command::HELO(read_line(&mut line).unwrap()))
            } else {
                Ok(Command::Invalid)
            }
        },
        Some('e') => {
            if line.match_next_bytes_ignore_case(b"HLO ") {
                Ok(Command::EHLO(read_line(&mut line).unwrap()))
            } else {
                Ok(Command::Invalid)
            }
        },
        Some('r') => {
            if line.match_next_bytes_ignore_case(b"CPT TO:") {
                Ok(Command::RCPT_TO(read_line(&mut line).unwrap()))
            } else {
                Ok(Command::Invalid)
            }
        },
        Some('d') => {
            if line.match_next_bytes_ignore_case(b"ATA\r\n") {
                Ok(Command::DATA)
            } else {
                Ok(Command::Invalid)
            }
        },
        Some('q') => {
            if line.match_next_bytes_ignore_case(b"UIT\r\n") {
                Ok(Command::QUIT)
            } else {
                Ok(Command::Invalid)
            }
        },
        _ => Ok(Command::Invalid)
    }
}

fn test_parse_command(input : &str, expected: Result<Command, ParseError>) {
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
