use std::io::Read;

use data::{ParseError};

// RFC 5321 Section 2.3.8. Lines
const CR: u8 = 0x0D;
const LF: u8 = 0x0A;

pub fn read_line(io: &mut Read) -> Result<String, ParseError> {
    let mut s = "".to_string();
    let mut buf : &mut [u8] = &mut [0];
    loop {
        if let Ok(_) = io.read_exact(buf) {
            match buf[0] {
                CR => break,
                LF => {
                    return Err(ParseError::SyntaxError("Expected CR (before LF). Got LF"));
                },
                byte => {
                    s.push(byte as char);
                }
            }
        }
    }

    if let Ok(_) = io.read_exact(&mut buf) {
        Ok(s)
    } else {
        Err(ParseError::InvalidLineEnding)
    }
}
