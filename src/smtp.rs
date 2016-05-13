use std::io::{Read, Write, Result, Error, ErrorKind};
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
        },
        Err(_) => {
            error!("IO error while reading command. Quitting");
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
        let cmd = read_command(&mut conn);
        match cmd {
            Ok(Command::MAIL_FROM(mailfrom)) => {
                println!("FROM: {}", mailfrom);
                bytes_to_write.extend("250 Ok\r\n".as_bytes().iter())
            },
            Ok(Command::RCPT_TO(mailto)) => {
                println!("TO: {}", mailto);
                bytes_to_write.extend("250 Ok\r\n".as_bytes().iter());
            },
            Ok(Command::DATA) => {
                println!("DATA");
                bytes_to_write.extend("354 End data with <CR><LF>.<CR><LF>\r\n".as_bytes().iter());
                loop {
                    let line = read_line(&mut conn).unwrap();
                    println!("Data|{}|", line);
                    match &line.as_str() {
                        &"." => {
                            println!("Got end");
                            break;
                        },
                        _ => {}
                    };
                }
                bytes_to_write.extend("250 Ok\r\n".as_bytes().iter());
            },
            Ok(Command::QUIT) => {
                println!("QUIT");
                bytes_to_write.extend("221 Bye\r\n".as_bytes().iter());
                break;
            },
            Ok(_) => panic!("Unknown command {:?}", cmd),
            Err(_) => panic!("IO Error"),
        };

        if let Ok(_) = conn.write_all(&bytes_to_write) {
            let flush_result = conn.flush();
            if !flush_result.is_ok() {
                error!("Failed to flush bytes to connection. Ending session");
                return;
            }
        } else {
            error!("Failed to write bytes. Ending session.");
            return;
        }

    }
}
