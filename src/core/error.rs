use std::error::Error;

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
            message: e.description().to_string(),
        }
    }
}

// impl std::convert::From<tiled::TiledError> for EmeraldError {
//     fn from(t: tiled::TiledError) -> EmeraldError {
//         EmeraldError {
//             message: t.description().to_string(),
//         }
//     }
// }