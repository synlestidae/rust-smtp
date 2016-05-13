use std::cmp::min;
use std;
use std::io::Read;

use data::{Command, ParseError};

fn ignore_ascii_case(byte_in: u8) -> u8 {
    let mut byte = byte_in;
    if 65 <= byte && byte <= 90 {
        byte += 32;
    } 
    byte
}

fn ascii_slice_eq_ignore_case(a_byte: &[u8], b_byte: &[u8]) -> bool {
    a_byte.len() == b_byte.len() && a_byte.iter()
        .zip(b_byte.iter())
        .skip_while(|&(&a, &b)| ascii_eq_ignore_case(a, b))
        .collect::<Vec<_>>()
        .len() == 0
}

fn ascii_eq_ignore_case(a_byte: u8, b_byte: u8) -> bool {
    ignore_ascii_case(a_byte) == ignore_ascii_case(b_byte)
}

fn read_expect(scanner: &SliceScanner<u8>, expect: &[u8]) -> bool {
        for (i, &byte) in expect.iter().enumerate() {
            if scanner.data[i] != byte {
                return false;
            }
        }
        return false;
    }

fn read_expect_ignore_case(scanner: &mut SliceScanner<u8>, expect: &[u8]) -> bool {
    let mut bytes_read = 0;

    for (i, &byte) in expect.iter().enumerate() {
        if !ascii_eq_ignore_case(byte, scanner.data[i]) {
            return false;
        }
        bytes_read = i;
    }

    scanner.pop_front(bytes_read);
    return true;
}

fn pop_front_byte_ignore_case(scanner: &mut SliceScanner<u8>) -> u8 {
    ignore_ascii_case(scanner.pop_front(1)[0])
}

// RFC 5321 Section 2.3.8. Lines
const CR: u8 = 0x0D;
const LF: u8 = 0x0A;

fn read_line(io: &mut SliceScanner<u8>) -> Result<String, ParseError> {
    let mut s = "".to_string();

    loop {
        match io.pop_front(1)[0] {
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

struct SliceScanner<'a, T: 'a> {
    data: &'a [T],
}

#[allow(dead_code)]
impl<'a, T> SliceScanner<'a, T> {
    fn new(data: &'a [T]) -> SliceScanner<'a, T> {
        SliceScanner { data: data }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn data(&self) -> &'a [T] {
        self.data
    }

    /// Remove `n` (but no more than len()) items from the back and return them.
    fn pop_back(&mut self, n: usize) -> &'a [T] {
        if n > self.len() {
            debug!("pop_back(): n > len");
        }
        let n = min(n, self.len());
        let (front, back) = self.split_at(self.len() - n);
        self.data = front;
        return back;
    }

    /// Remove `n` (but no more than len()) items from the front and return them. 
    fn pop_front(&mut self, n: usize) -> &'a [T] {
        if n > self.len() {
            debug!("pop_front(): n > len");
        }
        let n = min(n, self.len());

        let (front, back) = self.split_at(n);
        self.data = back;
        return front;
    }

    /// Same as pop_front, but does not modify the underlying SliceScanner.
    fn ref_front(&self, n: usize) -> &'a [T] {
        if n > self.len() {
            debug!("ref_front(): n > len");
        }
        let n = min(n, self.len());

        let (front, _) = self.split_at(n);
        return front;
    }

    fn count_while(&self, cond: fn(&T) -> bool) -> usize {
        let mut cnt = 0;
        for b in self.data.iter() {
            if cond(b) {
                cnt += 1;
            } else {
                break;
            }
        }
        return cnt;
    }

    fn pop_while(&mut self, cond: fn(&T) -> bool) -> &'a [T] {
        let cnt = self.count_while(cond);
        self.pop_front(cnt)
    }

    fn split_at(&self, pos: usize) -> (&'a [T], &'a [T]) {
        assert!(pos <= self.data.len());
        (&self.data[0..pos], &self.data[pos..self.data.len()])
    }
}

fn is_not_space_byte(b: &u8) -> bool {
    match b {
        &byte => byte != 32,
    }
}

fn is_space_byte(b: &u8) -> bool {
    match b {
        &byte => byte == 32,
    }
}

fn is_not_less_than(b: &u8) -> bool {
    match b {
        &byte => byte != ('>' as u8),
    }
}

pub fn read_command(reader: &mut Read) -> Result<Command, ParseError> {
    panic!("Not implemented")
}

fn parse_command(input_line: &[u8]) -> Result<Command, ParseError> {
    let mut line: SliceScanner<u8> = SliceScanner::new(input_line);

    let crlf = line.pop_back(2);
    if crlf != b"\r\n" {
        return Err(ParseError::InvalidLineEnding);
    }

    match pop_front_byte_ignore_case(&mut line) as char {
        'm' => {
            if line.pop_while(is_space_byte).len() == 0 {
                return Err(ParseError::SyntaxError("Invalid MAIL command: Missing space after MAIL"));
            }

            if ascii_slice_eq_ignore_case(line.pop_front(5), b"FROM:") {
                let addr = if line.ref_front(1) == b"<" {
                    let _ = line.pop_front(1);
                    let addr = line.pop_while(is_not_less_than);
                    if line.pop_front(1) != b">" {
                        return Err(ParseError::SyntaxError("Invalid MAIL command: Missing >"));
                    }
                    addr
                } else {
                    line.pop_while(is_not_space_byte)
                };

                if line.is_empty() {
                    // XXX: Verify mail addr
                    Ok(Command::MAIL_FROM(String::from_utf8(addr.iter().map(|&b| b).collect::<Vec<_>>()).unwrap()))
                } else {
                    println!("There were trailing characters on mail command: `{}`", std::str::from_utf8(line.data).unwrap());
                    Err(ParseError::SyntaxError("Invalid trailing characters on MAIL command"))
                }
            } else {
                Err(ParseError::SyntaxError("Invalid MAIL command"))
            }
        },
        'h' => {
            if read_expect_ignore_case(&mut line, b"ELO ") {
                Ok(Command::HELO(read_line(&mut line).unwrap()))
            } else {
                Ok(Command::Invalid)
            }
        },
        'e' => {
            if read_expect_ignore_case(&mut line, b"HLO ") {
                Ok(Command::EHLO(read_line(&mut line).unwrap()))
            } else {
                Ok(Command::Invalid)
            }
        },
        'r' => {
            if read_expect_ignore_case(&mut line, b"CPT TO:") {
                Ok(Command::RCPT_TO(read_line(&mut line).unwrap()))
            } else {
                Ok(Command::Invalid)
            }
        },
        'd' => {
            if read_expect_ignore_case(&mut line, b"ATA\r\n") {
                Ok(Command::DATA)
            } else {
                Ok(Command::Invalid)
            }
        },
        'q' => {
            if read_expect_ignore_case(&mut line, b"UIT\r\n") {
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
