use crate::logging::*;

pub(crate) enum Log {
    Info(String),
    Warning(String),
    Error(String),
}


pub(crate) struct LoggingEngine {
    logs: Vec<Log>,
}
impl LoggingEngine {
    pub(crate) fn new() -> Self { 
        LoggingEngine {
            logs: Vec::new(),
        }
    }

    pub fn update(&mut self) {
        for log in &self.logs {
            match log {
                Log::Info(msg) => println!("{}", msg),
                Log::Warning(msg) => println!("{}", msg),
                Log::Error(msg) => println!("{}", msg),
            }
        }

        self.logs = Vec::with_capacity(self.logs.len());
    }

    pub fn log(&mut self, log: Log) {
        self.logs.push(log);
    }
}