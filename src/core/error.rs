
#[derive(Debug, Clone)]
pub struct EmeraldError {
    message: String,
}
impl EmeraldError {
    pub fn new<T: Into<String>>(msg: T) -> Self {
        EmeraldError {
            message: msg.into()
        }
    }
}
impl std::fmt::Display for EmeraldError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::convert::From<nanoserde::DeJsonErr> for EmeraldError {
    fn from(e: nanoserde::DeJsonErr) -> EmeraldError {
        EmeraldError {
            message: e.to_string(),
        }
    }
}

impl std::convert::From<std::io::Error> for EmeraldError {
    fn from(e: std::io::Error) -> EmeraldError {
        EmeraldError {
            message: e.to_string(),
        }
    }
}
impl std::convert::From<&str> for EmeraldError {
    fn from(e: &str) -> EmeraldError {
        EmeraldError {
            message: e.to_string(),
        }
    }
}

impl std::convert::From<std::string::FromUtf8Error> for EmeraldError {
    fn from(e: std::string::FromUtf8Error) -> EmeraldError {
        EmeraldError {
            message: e.to_string()
        }
    }
}

impl std::convert::From<crossbeam::channel::TryRecvError> for EmeraldError {
    fn from(e: crossbeam::channel::TryRecvError) -> EmeraldError {
        EmeraldError {
            message: e.to_string()
        }
    }
}

impl std::convert::From<std::ffi::OsString> for EmeraldError {
    fn from(e: std::ffi::OsString) -> EmeraldError {
        EmeraldError {
            message: String::from("Unable to parse string out of OsString")
        }
    }
}
