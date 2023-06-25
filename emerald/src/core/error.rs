use crate::*;

#[derive(Debug, Clone)]
pub struct EmeraldError {
    pub message: String,
}
impl EmeraldError {
    pub fn new<T: Into<String>>(msg: T) -> Self {
        EmeraldError {
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for EmeraldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<serde_json::Error> for EmeraldError {
    fn from(e: serde_json::Error) -> EmeraldError {
        EmeraldError {
            message: format!("serde_json::Error {:?}", &e.to_string()),
        }
    }
}
impl From<toml::de::Error> for EmeraldError {
    fn from(e: toml::de::Error) -> EmeraldError {
        EmeraldError {
            message: format!("toml::de::Error {:?}", &e.to_string()),
        }
    }
}

impl From<image::ImageError> for EmeraldError {
    fn from(e: image::ImageError) -> EmeraldError {
        EmeraldError {
            message: format!("image::ImageError {:?}", &e.to_string()),
        }
    }
}

impl From<std::io::Error> for EmeraldError {
    fn from(e: std::io::Error) -> EmeraldError {
        EmeraldError {
            message: format!("std::io::Error {:?}", &e.to_string()),
        }
    }
}

impl From<std::str::Utf8Error> for EmeraldError {
    fn from(e: std::str::Utf8Error) -> EmeraldError {
        EmeraldError {
            message: format!("std::str::Utf8Error {:?}", &e.to_string()),
        }
    }
}

impl From<std::string::FromUtf8Error> for EmeraldError {
    fn from(e: std::string::FromUtf8Error) -> EmeraldError {
        EmeraldError {
            message: format!("std::string::FromUtf8Error {:?}", &e.to_string()),
        }
    }
}

impl From<rapier2d::crossbeam::channel::TryRecvError> for EmeraldError {
    fn from(e: rapier2d::crossbeam::channel::TryRecvError) -> EmeraldError {
        EmeraldError {
            message: format!("crossbeam::channel::TryRecvError {:?}", &e.to_string()),
        }
    }
}

impl From<std::ffi::OsString> for EmeraldError {
    fn from(_e: std::ffi::OsString) -> EmeraldError {
        EmeraldError {
            message: String::from("Unable to parse string out of OsString"),
        }
    }
}

impl From<hecs::NoSuchEntity> for EmeraldError {
    fn from(e: hecs::NoSuchEntity) -> EmeraldError {
        EmeraldError {
            message: format!("hecs::NoSuchEntity {:?}", &e.to_string()),
        }
    }
}

impl From<hecs::ComponentError> for EmeraldError {
    fn from(e: hecs::ComponentError) -> EmeraldError {
        EmeraldError {
            message: format!("hecs::ComponentError {:?}", &e.to_string()),
        }
    }
}

impl From<asefile::AsepriteParseError> for EmeraldError {
    fn from(e: asefile::AsepriteParseError) -> EmeraldError {
        use asefile::AsepriteParseError::*;

        let message = match e {
            InvalidInput(inner_msg) => format!("Invalid aseprite file: {}", inner_msg),
            UnsupportedFeature(inner_msg) => format!("Unsupported aseprite feature: {}", inner_msg),
            InternalError(inner_msg) => format!("Internal asefile error: {}", inner_msg),
            IoError(inner_err) => format!("IO error while reading aseprite file: {:?}", inner_err),
        };

        Self { message }
    }
}
