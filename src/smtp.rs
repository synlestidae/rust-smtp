use std::io::{Read, Write};
use data::Command;
use parser::read_command;
use parse_util::{read_line, read_line_bytes};
use smtp_state::{SmtpStateMachine, DefaultStateMachine, SmtpState, SmtpError};
use payload::Payload;
use std::sync::mpsc::Sender;

pub struct DefaultConnectionHandler {
    message_sender: Sender<Payload>,
}

impl DefaultConnectionHandler {
    pub fn new(message_sender: Sender<Payload>) -> DefaultConnectionHandler {
        DefaultConnectionHandler { message_sender: message_sender }
    }
}

pub trait ConnectionHandler {
    fn handle_connection<C: Read + Write>(&self, mut conn: C);
}

impl ConnectionHandler for DefaultConnectionHandler {
    fn handle_connection<C: Read + Write>(&self, mut conn: C) {
        debug!("Got connection");
        let server_hostname = "mail.ntecs.de";
        let server_agent = "rust-smtp";

        let response_220 = format!("220 {} ESMTP {}\r\n", server_hostname, server_agent);
        if let Err(_) = conn.write_all(&response_220.into_bytes()) {
            error!("Error while writing 220 hostname and agent response");
            return;
        }

        let client_hostname = match read_command(&mut conn) {
            Ok(Command::EHLO(h)) => h,
            Ok(unexpected) => {
                error!("Unexpected command {:?}", unexpected);
                return;
            }
            Err(error) => {
                error!("Error while reading command: {:?}. Quitting", error);
                return;
            }
        };

        info!("Client hostname: {}", client_hostname);

        if let Ok(_) = conn.write_all(&format!("250 Hello {}\r\n", client_hostname).into_bytes()) {
            info!("Saying Hello to {}", client_hostname);
        } else {
            error!("Error while writing Hello. Quitting session.");
            return;
        }

        let mut bytes_to_write: Vec<u8> = Vec::new();
        let mut session_state = DefaultStateMachine::new();

        loop {
            bytes_to_write.clear();
            let cmd_result = read_command(&mut conn);
            if let Ok(cmd) = cmd_result {
                if let Ok(response) = session_state.transition(&cmd) {
                    bytes_to_write.extend(response.to_bytes());
                } else {
                    // meh
                }
                _flush_bytes(&bytes_to_write, &mut conn);
            } else {
                bytes_to_write.extend(format!("500 Error while parsing command: {:?}\r\n",
                                              cmd_result)
                                          .bytes())
            }
            _handle_state(&session_state.state(), &mut conn);
        }
    }
}

fn _handle_state<C: Read + Write>(state: &SmtpState, conn: &mut C) {
    let mut waiting_for_fullstop = false;

    fn is_str_equal(bytes: &[u8], string: &str) -> bool {
        if bytes.len() != string.len() {
            false;
        }
        string.as_bytes() == bytes
    }

    match state {
        &SmtpState::ReadyForData => {
            let mut data = Vec::new();
            loop {
                let line_res = read_line_bytes(conn);
                if !line_res.is_ok() {
                    // TODO handle this better please!
                    return;
                }
                let line = line_res.unwrap();
                if (!waiting_for_fullstop) {
                    if is_str_equal(&line, "\r\n") {
                        waiting_for_fullstop = true;
                    } else {
                        data.extend(line);
                    }
                } else {
                    if is_str_equal(&line, ".\r\n") {
                        return;
                    } else {
                        data.push('\r' as u8);
                        data.push('\n' as u8);
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
