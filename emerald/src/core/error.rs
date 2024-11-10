use ::core::str::Utf8Error;

use alloc::{
    format,
    string::{FromUtf8Error, String, ToString},
};

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

impl From<Utf8Error> for EmeraldError {
    fn from(value: Utf8Error) -> Self {
        EmeraldError {
            message: format!("serde_json::Error {:?}", &value.to_string()),
        }
    }
}

// FromResidual<Result<Infallible, FromUtf8Error>>

impl From<serde_json::Error> for EmeraldError {
    fn from(e: serde_json::Error) -> EmeraldError {
        EmeraldError {
            message: format!("serde_json::Error {:?}", &e.to_string()),
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
