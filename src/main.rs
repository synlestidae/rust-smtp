use std::net::{TcpListener, TcpStream};
use log::{LogLevel, LogLevelFilter, LogRecord, LogMetadata};    
use std::thread::spawn;
use smtp;

pub fn main() {
    log::set_logger(|max_log_level| {
        max_log_level.set(LogLevelFilter::Info);
        Box::new(SimpleLogger)
    })
        .unwrap();

    match TcpListener::bind(("127.0.0.1", 2525)) {
        Ok(listener) => {
            for acceptor in listener.incoming() {
                match acceptor {
                    Ok(conn) => {
                        spawn(|| smtp::handle_connection(conn));
                    }
                    _ => (),
                }
            }
        }
        _ => panic!(),
    }
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }
}