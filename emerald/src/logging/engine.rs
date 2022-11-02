use crate::EmeraldError;

use std::{
    fs::{File, OpenOptions},
    io::{prelude::*, LineWriter},
};

pub(crate) enum Log {
    Info(String),
    Warning(String),
    Error(String),
}

const DEFAULT_LOG_FILE_PATH: &str = "./emerald.log";

pub struct LoggingEngine {
    logs: Vec<Log>,
    line_writer: Option<LineWriter<File>>,
}
impl LoggingEngine {
    pub(crate) fn new() -> Self {
        let mut line_writer = None;

        #[cfg(not(target_arch = "wasm32"))]
        {
            let file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(String::from(DEFAULT_LOG_FILE_PATH))
                .unwrap();
            line_writer = Some(LineWriter::new(file));
        }

        LoggingEngine {
            logs: Vec::new(),
            line_writer,
        }
    }

    pub(crate) fn update(&mut self) -> Result<(), EmeraldError> {
        if let Some(line_writer) = &mut self.line_writer {
            for log in &self.logs {
                match log {
                    Log::Info(msg) => line_writer.write_all(msg.as_bytes())?,
                    Log::Warning(msg) => line_writer.write_all(msg.as_bytes())?,
                    Log::Error(msg) => line_writer.write_all(msg.as_bytes())?,
                };

                line_writer.write_all("\n".as_bytes())?;
            }
            line_writer.flush()?;
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
