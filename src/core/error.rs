use kira::{CommandError, manager::error::{AddSoundError, SetupError}};

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
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::convert::From<image::ImageError> for EmeraldError {
    fn from(e: image::ImageError) -> EmeraldError {
        EmeraldError {
            message: e.to_string(),
        }
    }
}
impl std::convert::From<CommandError> for EmeraldError {
    fn from(e: CommandError) -> EmeraldError {
        EmeraldError {
            message: e.to_string(),
        }
    }
}
impl std::convert::From<SetupError> for EmeraldError {
    fn from(e: SetupError) -> EmeraldError {
        EmeraldError {
            message: e.to_string(),
        }
    }
}
impl std::convert::From<AddSoundError> for EmeraldError {
    fn from(e: AddSoundError) -> EmeraldError {
        EmeraldError {
            message: e.to_string(),
        }
    }
}

impl std::convert::From<kira::sound::error::SoundFromFileError> for EmeraldError {
    fn from(e: kira::sound::error::SoundFromFileError) -> EmeraldError {
        EmeraldError {
            message: e.to_string(),
        }
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
            message: e.to_string(),
        }
    }
}

#[cfg(feature = "physics")]
impl std::convert::From<crossbeam::channel::TryRecvError> for EmeraldError {
    fn from(e: crossbeam::channel::TryRecvError) -> EmeraldError {
        EmeraldError {
            message: e.to_string(),
        }
    }
}

impl std::convert::From<std::ffi::OsString> for EmeraldError {
    fn from(_e: std::ffi::OsString) -> EmeraldError {
        EmeraldError {
            message: String::from("Unable to parse string out of OsString"),
        }
    }
}

impl std::convert::From<hecs::NoSuchEntity> for EmeraldError {
    fn from(e: hecs::NoSuchEntity) -> EmeraldError {
        EmeraldError {
            message: e.to_string(),
        }
    }
}

impl std::convert::From<hecs::ComponentError> for EmeraldError {
    fn from(e: hecs::ComponentError) -> EmeraldError {
        EmeraldError {
            message: e.to_string(),
        }
    }
}

#[cfg(feature = "gamepads")]
impl std::convert::From<gamepad::GamepadError> for EmeraldError {
    fn from(e: gamepad::GamepadError) -> EmeraldError {
        EmeraldError { message: e.msg }
    }
}
