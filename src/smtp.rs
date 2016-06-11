use std::io;
use std::io::{Read, Write};
use data::Command;
use parser::read_command;
use parse_util::read_line_bytes;
use smtp_state::{SmtpStateMachine, DefaultStateMachine, SmtpState};
use payload::Payload;
use std::sync::mpsc::Sender;
use smtp_error::SmtpError;
use response::Response;

pub struct DefaultConnectionHandler {
    message_sender: Sender<Payload>,
}

impl DefaultConnectionHandler {
    pub fn new(message_sender: Sender<Payload>) -> DefaultConnectionHandler {
        DefaultConnectionHandler { message_sender: message_sender }
    }

    fn _say_hello_and_start_session<C: Read + Write, S: SmtpStateMachine>
        (&self,
         conn: &mut C)
         -> Result<S, SmtpError> {
        let server_hostname = "mail.ntecs.de";
        let server_agent = "rust-smtp";

        let response_220 = Response::new(220,
                                         &format!("220 {} ESMTP {}",
                                                  server_hostname,
                                                  server_agent));

        if let Err(err) = conn.write_all(&response_220.to_bytes()) {
            error!("Error while writing 220 hostname and agent response");
            return Err(SmtpError::IOError(err));
        }

        let client_hostname = match read_command(conn) {
            Ok(Command::EHLO(h)) => h,
            Ok(Command::HELO(h)) => h,
            Ok(unexpected_command) => {
                error!("Unexpected command {:?}", unexpected_command);
                return Err(SmtpError::UnexpectedCommand(unexpected_command,
                                                        "Expected EHLO or HELO"));
            }
            Err(error) => {
                error!("Error while reading command: {:?}. Quitting", error);
                return Err(SmtpError::ParseError(error));
            }
        };

        info!("Client hostname: {}", client_hostname);
        let hello_response = Response::new(250, &format!("Hello {}", client_hostname));
        let hello_result = conn.write_all(&hello_response.to_bytes());
        if let Ok(_) = hello_result {
            info!("Said Hello to {}", client_hostname);
        } else {
            error!("Error while writing Hello. Quitting session.");
            return Err(SmtpError::IOError(hello_result.err().unwrap()));
        }

        Ok(S::new())
    }
}

pub trait ConnectionHandler {
    fn handle_connection<C: Read + Write>(&self, mut conn: &mut C);
}

impl ConnectionHandler for DefaultConnectionHandler {
    fn handle_connection<C: Read + Write>(&self, conn: &mut C) {
        debug!("Got connection");

        let setup_result = self._say_hello_and_start_session::<C, DefaultStateMachine>(conn);
        if !setup_result.is_ok() {
            return;
        }
        let mut session_state = setup_result.unwrap();

        let mut bytes_to_write: Vec<u8> = Vec::new();
        loop {
            bytes_to_write.clear();
            let cmd_result = read_command(conn);
            if let Ok(cmd) = cmd_result {
                if let Ok(response) = session_state.transition(&cmd) {
                    bytes_to_write.extend(response.to_bytes());
                } else {
                    // meh
                }
                _flush_bytes(&bytes_to_write, conn);
            } else {
                bytes_to_write.extend(format!("500 Error while parsing command: {:?}\r\n",
                                              cmd_result)
                                          .bytes())
            }
            let current_state = session_state.state();

            if current_state == SmtpState::Quit {
                info!("Quitting now. Session will disconnect.");
                let payload = session_state.extract_payload();
                self.message_sender.send(payload);
                return;
            }
            _handle_state(&mut session_state, conn);
        }
    }
}

fn _handle_state<C: Read + Write, S: SmtpStateMachine>(state_machine: &mut S, conn: &mut C) {
    let mut waiting_for_fullstop = false;

    fn is_str_equal(bytes: &[u8], string: &str) -> bool {
        if bytes.len() != string.len() {
            false;
        }
        string.as_bytes() == bytes
    }

    match state_machine.state() {
        SmtpState::DataInProgress => {
            let mut data = Vec::new();
            loop {
                let line_res = read_line_bytes(conn);
                if !line_res.is_ok() {
                    // TODO handle this better please!
                    return;
                }
                let line = line_res.unwrap();
                if !waiting_for_fullstop {
                    data.extend(line);
                    waiting_for_fullstop = true;
                } else {
                    if is_str_equal(&line, ".\r\n") {
                        let payload = state_machine.get_payload_mut_ref();
                        payload.data = data;
                        return;
                    } else {
                        data.extend(line);
                        waiting_for_fullstop = false;
                    }
                }
            }
        }
        _ => (),
    }
}

fn _flush_bytes(bytes_to_write: &Vec<u8>, conn: &mut Write) {
    if let Ok(_) = conn.write_all(&bytes_to_write) {
        let flush_result = conn.flush();
        if !flush_result.is_ok() {
            error!("Failed to flush bytes to connection.");
            return;
        }
    } else {
        error!("Failed to write bytes.");
        return;
    }
}

pub mod tests {
    use std::io;
    use std::io::{Read, Write};
    use std::sync::mpsc::channel;
    use std::cmp::max;
    use smtp::{DefaultConnectionHandler, ConnectionHandler};
    use address::Address;

    struct MockStream {
        pub data_in: Vec<u8>,
        pub index: usize,
        pub data_out: Vec<u8>,
    }

    impl MockStream {
        pub fn new_session(session_string: &str) -> MockStream {
            MockStream {
                data_in: session_string.bytes().collect::<Vec<u8>>(),
                index: 0,
                data_out: Vec::new(),
            }
        }
    }

    impl Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.index >= self.data_in.len() {
                return Ok(0);
            }
            let index_end = if self.index + buf.len() > self.data_in.len() {
                self.data_in.len()
            } else {
                self.index + buf.len()
            };

            for (i, &b) in self.data_in[self.index..index_end].iter().enumerate() {
                buf[i] = b;
            }

            let old_index = self.index;
            self.index = index_end;

            Ok(self.index - old_index)
        }
    }

    impl Write for MockStream {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            let response = String::from_utf8(buf.iter().map(|&b| b).collect::<Vec<u8>>()).unwrap();
            self.data_out.extend(buf.iter().clone());
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    pub fn parses_basic_session() {
        let (payload_tx, payload_rx) = channel();
        let handler = DefaultConnectionHandler::new(payload_tx);
        let mut stream = MockStream::new_session("EHLO localhost\r\nMAIL FROM: \
                                                  matt@localhost\r\nRCPT TO: \
                                                  marie@localhost\r\nDATA\r\nHi \
                                                  Marie\r\n.\r\nQUIT\r\n");
        handler.handle_connection(&mut stream);
        assert!(payload_rx.try_recv().is_ok());
    }

#[test]
    pub fn parses_basic_session_2() {
        let (payload_tx, payload_rx) = channel();
        let handler = DefaultConnectionHandler::new(payload_tx);
        let mut stream = MockStream::new_session("HELO antunovic.nz\r\nMAIL FROM: mate@antunovic.nz\r\nRCPT TO: just.mate.antunovic@gmail.com\r\nDATA\r\nHello, how are ya\r\n.\r\nQUIT\r\n");
        handler.handle_connection(&mut stream);
        let mut string_bytes = Vec::new();
        stream.read_to_end(&mut string_bytes);
        let payload = payload_rx.try_recv().ok().unwrap();
        println!("Session: {}", String::from_utf8(string_bytes).unwrap());
        assert_eq!(Some(Address::new("mate","antunovic.nz")), payload.sender);
        assert_eq!(vec![Address::new("just.mate.antunovic","gmail.com")], payload.recipients);
        assert_eq!("Hello, how are ya\r\n".to_string().into_bytes(), payload.data);
    }

}
