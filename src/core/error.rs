
#[derive(Debug, Clone)]
pub struct EmeraldError {
    message: String,
}
impl std::fmt::Display for EmeraldError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
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
