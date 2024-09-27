use env_logger;
use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
use serde::Serialize;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

pub struct EnvTcpLogger {
    hostname: String,
    env_logger: env_logger::Logger,
    log_tx: Sender<String>,
}

#[derive(Serialize)]
struct SocketMessage {
    hostname: String,
    level: String,
    message: String,
    module: String,
}

impl EnvTcpLogger {
    pub fn init(
        hostname: String,
        tcp_address: String,
        env_logger: env_logger::Logger,
    ) -> Result<(), SetLoggerError> {
        let (log_tx, log_rx) = mpsc::channel();
        let logger = EnvTcpLogger {
            hostname,
            env_logger,
            log_tx,
        };
        std::thread::spawn(move || {
            retry_setup_socket(tcp_address, log_rx);
        });
        log::set_boxed_logger(Box::new(logger))?;
        log::set_max_level(LevelFilter::Trace);
        Ok(())
    }
}

impl Log for EnvTcpLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.env_logger.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        // Forward the log to env_logger.
        self.env_logger.log(record);
        // Forward the log to TCP socket.
        let serialization_result = serde_json::to_string(&SocketMessage {
            hostname: self.hostname.clone(),
            level: record.level().to_string(),
            message: record.args().to_string(),
            module: record.target().to_string()
        });
        let message = match serialization_result {
            Ok(message) => message,
            Err(error) => {
                eprintln!("failed to serialize TCP socket message: {error}");
                return;
            }
        };
        let result = self.log_tx.send(message);
        if let Err(error) = result {
            eprintln!("error sending TCP socket channel message: {error}");
        }
    }

    fn flush(&self) {
        self.env_logger.flush();
    }
}

fn retry_setup_socket(tcp_address: String, mut log_rx: Receiver<String>) {
    loop {
        let result = setup_socket(&tcp_address, &mut log_rx);
        if let Err(error) = result {
            eprintln!("TCP socket log error: {error}");
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn setup_socket(tcp_address: &str, log_rx: &mut Receiver<String>) -> std::io::Result<()> {
    let mut socket = TcpStream::connect(tcp_address)?;
    for message in log_rx.iter() {
        let framed_message = format!("{message}\0");
        socket.write_all(framed_message.as_bytes())?;
        socket.flush()?;
    }
    Ok(())
}
