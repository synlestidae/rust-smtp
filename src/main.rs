extern crate nibbler;
extern crate log;

use std::net::TcpListener;
use std::thread::spawn;
use std::error::Error;
use nibbler::smtp::{DefaultConnectionHandler, ConnectionHandler};
use std::sync::mpsc::{Sender, Receiver, channel};

pub fn main() {
    log::set_logger(|max_log_level| {
        max_log_level.set(log::LogLevelFilter::Info);
        Box::new(SimpleLogger)
    })
        .unwrap();

    let (message_tx, message_rx) = channel();

    spawn(move || {
        loop {
            if let Ok(message) = message_rx.recv() {
                println!("Got message.");
            }
        }
    });

    match TcpListener::bind(("127.0.0.1", 25255)) {
        Ok(listener) => {
            for acceptor in listener.incoming() {
                let handler = DefaultConnectionHandler::new(message_tx.clone());
                match acceptor {
                    Ok(conn) => {
                        spawn(move || handler.handle_connection(conn));
                    }
                    _ => (),
                }
            }
        }
        Err(error) => panic!("Could not bind server to socket: {}", error.description()),
    }
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::LogMetadata) -> bool {
        metadata.level() <= log::LogLevel::Info
    }

    fn log(&self, record: &log::LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
}
