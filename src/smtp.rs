use std::io::{Read, Write};
use data::Command;
use parser::read_command;
use parse_util::read_line;

pub fn handle_connection<C: Read + Write>(mut conn: C) {
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

    println!("Client hostname: {}", client_hostname);

    if let Ok(_) = conn.write_all(&format!("250 Hello {}\r\n", client_hostname).into_bytes()) {
        info!("Saying Hello to {}", client_hostname);
    } else {
        error!("Error while writing Hello. Quitting session.");
        return;
    }

    let mut bytes_to_write: Vec<u8> = Vec::new();
    loop {
        bytes_to_write.clear();
        let cmd_result = read_command(&mut conn);
        if let Ok(cmd) =  cmd_result {
            handle_command_behaviour(&cmd, &mut bytes_to_write, &mut conn);
            flush_bytes(&bytes_to_write, &mut conn);
        }
        else {
            bytes_to_write.extend(format!("500 Error while parsing command: {:?}\r\n", cmd_result).bytes())
        }
    }
}

fn flush_bytes(bytes_to_write: &Vec<u8>, conn: &mut Write) {
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

fn handle_command_behaviour(cmd: &Command, bytes_to_write: &mut Vec<u8>, conn: &mut Read) {
    match cmd {
        &Command::MAIL_FROM(ref mailfrom) => {
            println!("FROM: {}", mailfrom);
            bytes_to_write.extend("250 Ok\r\n".as_bytes().iter())
        },
        &Command::RCPT_TO(ref mailto) => {
            println!("TO: {}", mailto);
            bytes_to_write.extend("250 Ok\r\n".as_bytes().iter());
        },
        &Command::DATA => {
            println!("DATA");
            bytes_to_write.extend("354 End data with <CR><LF>.<CR><LF>\r\n".as_bytes().iter());
            loop {
                let line = read_line(conn).unwrap();
                println!("Data|{}|", line);
                match &line.as_str() {
                    &"." => {
                        println!("Got end");
                        break;
                    }
                    _ => {}
                };
            }
            bytes_to_write.extend("250 Ok\r\n".as_bytes().iter());
        },
        &Command::QUIT => {
            println!("QUIT");
            bytes_to_write.extend("221 Bye\r\n".as_bytes().iter());
        },
        command => bytes_to_write.extend(format!("Unknown command {:?}", command).bytes())
    }
}