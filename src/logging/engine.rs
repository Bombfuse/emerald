use crate::EmeraldError;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::LineWriter;

pub(crate) enum Log {
    Info(String),
    Warning(String),
    Error(String),
}

const DEFAULT_LOG_FILE_PATH: &str = "./emerald.log";

pub struct LoggingEngine {
    logs: Vec<Log>,
    log_file_path: String,
    line_writer: LineWriter<File>,
}
impl LoggingEngine {
    pub(crate) fn new() -> Self {
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(String::from(DEFAULT_LOG_FILE_PATH))
            .unwrap();

        LoggingEngine {
            logs: Vec::new(),
            log_file_path: String::from(DEFAULT_LOG_FILE_PATH),
            line_writer: LineWriter::new(file),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn update(&mut self) -> Result<(), EmeraldError> {
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn update(&mut self) -> Result<(), EmeraldError> {
        #[cfg(not(debug_assertions))]
        {
            for log in &self.logs {
                match log {
                    Log::Info(msg) => self.line_writer.write_all(msg.as_bytes())?,
                    Log::Warning(msg) => self.line_writer.write_all(msg.as_bytes())?,
                    Log::Error(msg) => self.line_writer.write_all(msg.as_bytes())?,
                };

                self.line_writer.write_all("\n".as_bytes())?;
            }
            self.line_writer.flush()?;
        }

        #[cfg(debug_assertions)]
        {
            for log in &self.logs {
                match log {
                    Log::Info(msg) => println!("{}", msg),
                    Log::Warning(msg) => println!("{}", msg),
                    Log::Error(msg) => println!("{}", msg),
                };
            }
        }

        self.logs = Vec::with_capacity(self.logs.len());
        Ok(())
    }

    fn log(&mut self, log: Log) {
        self.logs.push(log);
    }

    pub fn info<T: Into<String>>(&mut self, msg: T) -> Result<(), EmeraldError> {
        let log = Log::Info(msg.into());

        self.log(log);
        self.update()?;

        Ok(())
    }

    pub fn warning<T: Into<String>>(&mut self, msg: T) -> Result<(), EmeraldError> {
        let log = Log::Warning(msg.into());

        self.log(log);
        self.update()?;

        Ok(())
    }

    pub fn error<T: Into<String>>(&mut self, msg: T) -> Result<(), EmeraldError> {
        let log = Log::Error(msg.into());

        self.log(log);
        self.update()?;

        Ok(())
    }
}
