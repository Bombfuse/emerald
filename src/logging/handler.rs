use crate::logging::*;

pub struct LoggingHandler<'a> {
    engine: &'a mut LoggingEngine,
    messages: Vec<Log>,
}
impl<'a> LoggingHandler<'a> {
    pub(crate) fn new(engine: &'a mut LoggingEngine) -> Self {
        LoggingHandler {
            engine,
            messages: Vec::new(),
        }
    }

    pub fn info<T: Into<String>>(&mut self, message: T) {
        let log = Log::Info(message.into());
        
        self.engine.log(log);
    }

    pub fn error<T: Into<String>>(&mut self, message: T) {
        let log = Log::Error(message.into());
        self.engine.log(log);
    }
}