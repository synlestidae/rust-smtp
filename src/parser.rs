use std::cmp::min;
use std;

#[allow(dead_code)]
#[derive(Eq, PartialEq, Debug)]
enum SmtpCommand<'a> {
    MAIL(&'a str),
    RCPT(&'a str),
    DATA,
}

#[allow(dead_code)]
#[derive(Eq, PartialEq, Debug)]
enum ParseError {
    SyntaxError(&'static str),
    InvalidLineEnding,
    UnknownCommand,
}

fn ascii_eq_ignore_case(a: &[u8], b: &[u8]) -> bool {
    if a.len() == b.len() {
        for (i, &a_byte) in a.iter().enumerate() {
            let b_byte = b[i];
            if a_byte < 127 && b_byte < 127 {
                if a_byte != b_byte {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    } else {
        false
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
        &byte => byte == (' ' as u8),
    }
}
fn is_not_less_than(b: &u8) -> bool {
    match b {
        &byte => byte != ('>' as u8),
    }
}


fn parse_command<'a>(line: &'a [u8]) -> Result<SmtpCommand<'a>, ParseError> {
    let mut line = SliceScanner::new(line);

    let crlf = line.pop_back(2);
    if crlf != b"\r\n" {
        return Err(ParseError::InvalidLineEnding);
    }

    let cmd = line.pop_front(4);
    if ascii_eq_ignore_case(cmd, b"MAIL") {

        if line.pop_while(is_not_space_byte).len() == 0 {
            return Err(ParseError::SyntaxError("Invalid MAIL command: Missing SP"));
        }

        if ascii_eq_ignore_case(line.pop_front(5), b"FROM:") {
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
                Ok(SmtpCommand::MAIL(std::str::from_utf8(addr).unwrap()))
            } else {
                Err(ParseError::SyntaxError("Invalid MAIL command"))
            }
        } else {
            Err(ParseError::SyntaxError("Invalid MAIL command"))
        }
    } else if ascii_eq_ignore_case(cmd, b"DATA") {
        if line.is_empty() {
            Ok(SmtpCommand::DATA)
        } else {
            Err(ParseError::SyntaxError("Invalid DATA command"))
        }
    } else {
        Err(ParseError::UnknownCommand)
    }
}

macro_rules! assert_match(
    ($given:expr , $pattern:pat) => ({
        match $given {
          $pattern => {}
          _ => {
             panic!("assertion failed: `value {:?} does not match pattern`", $given);
          }
        }
    })
);

macro_rules! test_parse_command (
  ($str:expr, $pat:pat) => (
    println!("Parsing `{}`", $str);
    assert_match!(parse_command(&($str.to_string().into_bytes())), $pat);
// match $str.to_string().to_bytes() {
//  cmd => assert_match!(parse_command(cmd), $pat)
// }
  )
);

#[test]
fn test_commands() {
    // test_parse_command!("", Err(InvalidLineEnding));
    test_parse_command!("\r", Err(ParseError::InvalidLineEnding));
    test_parse_command!("\n", Err(ParseError::InvalidLineEnding));
    test_parse_command!("\n\r", Err(ParseError::InvalidLineEnding));
    test_parse_command!("MAIL FROM:<mneumann@ntecs.de>",
                        Err(ParseError::InvalidLineEnding));
    test_parse_command!("MAIL FROM:<mneumann@ntecs.de>\r",
                        Err(ParseError::InvalidLineEnding));
    test_parse_command!("MAIL FROM:<mneumann@ntecs.de>\n",
                        Err(ParseError::InvalidLineEnding));
    test_parse_command!("MAIL FROM:<mneumann@ntecs.de>\n\r",
                        Err(ParseError::InvalidLineEnding));

    test_parse_command!("MAIL FROM:<mneumann@ntecs.de blah\r\n",
                        Err(ParseError::SyntaxError("Invalid MAIL command: Missing >")));

    test_parse_command!("MAIL FROM:<mneumann@ntecs.de>\r\n",
                        Ok(SmtpCommand::MAIL("mneumann@ntecs.de")));
    test_parse_command!("MAIL FROM:mneumann@ntecs.de\r\n",
                        Ok(SmtpCommand::MAIL("mneumann@ntecs.de")));


    test_parse_command!("DATA\r\n", Ok(SmtpCommand::DATA));
    test_parse_command!("data\r\n", Ok(SmtpCommand::DATA));
    test_parse_command!("data test\r\n",
                        Err(ParseError::SyntaxError("Invalid DATA command")));
}

fn main() {
    let buf = b"MAIL FROM:<mneumann@ntecs.de>\r\n";
    let cmd = parse_command(buf);
    println!("{:?}", cmd);
}
