use std::io::Read;
use data::ParseError;

// RFC 5321 Section 2.3.8. Lines
pub const CR: u8 = 0x0D;
pub const LF: u8 = 0x0A;

#[derive(Debug)]
pub struct SliceScanner<'a> {
    data: &'a [u8],
    index: usize,
}

impl<'a> SliceScanner<'a> {
    pub fn new(data: &'a [u8]) -> SliceScanner {
        SliceScanner {
            data: data,
            index: 0,
        }
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.index < self.data.len() {
            self.index = self.index + 1;
            Some(self.data[self.index - 1])
        } else {
            None
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.data.len() <= self.index
    }

    pub fn pop_while<P>(&mut self, predicate: P) -> Vec<u8>
        where P: Fn(u8) -> bool
    {
        let mut result = Vec::new();
        while self.index < self.data.len() {
            match self.data[self.index] {
                b => {
                    if predicate(b) {
                        result.push(b);
                        self.index = self.index + 1;
                    } else {
                        break;
                    }
                }
            };
        }
        result
    }

    pub fn match_next_bytes_ignore_case(&mut self, expected_bytes: &[u8]) -> bool {
        if self.index + expected_bytes.len() > self.data.len() {
            println!("Expect string too loong self.index={}, expected_bytes={}, self.data={}",
                     self.index,
                     expected_bytes.len(),
                     self.data.len());
            return false;
        }

        let expected_len = expected_bytes.len();

        let is_match = (&self.data[self.index..(self.index + expected_len)])
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

    pub fn read_line(&mut self) -> Result<String, ParseError> {
        let mut s = "".to_string();

        loop {
            match self.pop() {
                Some(CR) => break,
                Some(LF) => {
                    return Err(ParseError::SyntaxError("Expected CR (before LF). Got LF"));
                }
                Some(byte) => {
                    s.push(byte as char);
                }
                None => return Err(ParseError::InvalidLineEnding),
            }
        }

        if self.pop() == Some(LF as u8) {
            Ok(s)
        } else {
            Err(ParseError::InvalidLineEnding)
        }
    }
}

pub fn read_line(stream: &mut Read) -> Result<String, ParseError> {
    let mut s = "".to_string();
    let mut buf = vec![0];
    let mut ready_for_lf = false;
    loop {
        match stream.read(&mut buf) {
            Ok(_) => {
                if buf[0] == CR {
                    s.push('\r');
                    ready_for_lf = true;
                } else if buf[0] == LF {
                    if ready_for_lf {
                        s.push('\n');
                        return Ok(s);
                    } else {
                        return Err(ParseError::InvalidLineEnding);
                    }

                } else {
                    ready_for_lf = false;
                    s.push(buf[0] as char);
                }
            }
            Err(_) => return Err(ParseError::InvalidLineEnding),
        }
    }
}

pub fn ascii_eq_ignore_case(a_byte: u8, b_byte: u8) -> bool {
    ignore_ascii_case(a_byte) == ignore_ascii_case(b_byte)
}

pub fn is_space_byte(byte: u8) -> bool {
    byte == 32
}

pub fn ignore_ascii_case(byte_in: u8) -> u8 {
    let mut byte = byte_in;
    if 65 <= byte && byte <= 90 {
        byte += 32;
    }
    byte
}
