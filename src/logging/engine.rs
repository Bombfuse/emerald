use crate::EmeraldError;

#[cfg(target_arch = "wasm32")]
use miniquad::{error, info, warn};

#[cfg(all(
    feature = "logging",
    not(target_arch = "wasm32"),
    not(debug_assertions)
))]
use std::{
    fs::{File, OpenOptions},
    io::{prelude::*, LineWriter},
};

#[cfg(feature = "logging")]
pub(crate) enum Log {
    Info(String),
    Warning(String),
    Error(String),
}

#[cfg(all(
    feature = "logging",
    not(target_arch = "wasm32"),
    not(debug_assertions)
))]
const DEFAULT_LOG_FILE_PATH: &str = "./emerald.log";

pub struct LoggingEngine {
    #[cfg(feature = "logging")]
    logs: Vec<Log>,

    #[cfg(all(
        feature = "logging",
        not(target_arch = "wasm32"),
        not(debug_assertions)
    ))]
    line_writer: LineWriter<File>,
}
impl LoggingEngine {
    pub(crate) fn new() -> Self {
        #[cfg(all(
            feature = "logging",
            not(target_arch = "wasm32"),
            not(debug_assertions)
        ))]
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(String::from(DEFAULT_LOG_FILE_PATH))
            .unwrap();

        LoggingEngine {
            #[cfg(feature = "logging")]
            logs: Vec::new(),

            #[cfg(all(
                feature = "logging",
                not(target_arch = "wasm32"),
                not(debug_assertions)
            ))]
            line_writer: LineWriter::new(file),
        }
    }

    #[cfg(not(feature = "logging"))]
    pub(crate) fn update(&mut self) -> Result<(), EmeraldError> {
        Ok(())
    }

    #[cfg(feature = "logging")]
    pub(crate) fn update(&mut self) -> Result<(), EmeraldError> {
        #[cfg(all(not(debug_assertions), not(target_arch = "wasm32")))]
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

        #[cfg(all(not(debug_assertions), target_arch = "wasm32"))]
        {
            for log in &self.logs {
                match log {
                    Log::Info(msg) => info!("{}", msg),
                    Log::Warning(msg) => warn!("{}", msg),
                    Log::Error(msg) => error!("{}", msg),
                };
            }
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

    #[cfg(feature = "logging")]
    fn log(&mut self, log: Log) {
        self.logs.push(log);
    }

    #[cfg(feature = "logging")]
    pub fn info<T: Into<String>>(&mut self, msg: T) -> Result<(), EmeraldError> {
        let log = Log::Info(msg.into());

        self.log(log);
        self.update()?;

        Ok(())
    }

    #[cfg(feature = "logging")]
    pub fn warning<T: Into<String>>(&mut self, msg: T) -> Result<(), EmeraldError> {
        let log = Log::Warning(msg.into());

        self.log(log);
        self.update()?;

        Ok(())
    }

    #[cfg(feature = "logging")]
    pub fn error<T: Into<String>>(&mut self, msg: T) -> Result<(), EmeraldError> {
        let log = Log::Error(msg.into());

        self.log(log);
        self.update()?;

        Ok(())
    }

    #[cfg(not(feature = "logging"))]
    pub fn info<T: Into<String>>(&mut self, _msg: T) -> Result<(), EmeraldError> {
        Ok(())
    }

    #[cfg(not(feature = "logging"))]
    pub fn warning<T: Into<String>>(&mut self, _msg: T) -> Result<(), EmeraldError> {
        Ok(())
    }

    #[cfg(not(feature = "logging"))]
    pub fn error<T: Into<String>>(&mut self, _msg: T) -> Result<(), EmeraldError> {
        Ok(())
    }
}
