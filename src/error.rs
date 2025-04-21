use std::fmt::{Display, Formatter};
use std::io::Error as IoError;

use nokhwa::NokhwaError;

#[derive(Debug)]
pub enum Error {
    OnnxModelFileNotFound,
    InvalidJSONConfgiFile,
    CameraFailed,
    OtherError(anyhow::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::OnnxModelFileNotFound => write!(f, "ONNX model file not found"),
            Error::InvalidJSONConfgiFile => write!(f, "Invalid JSON config file"),
            Error::CameraFailed => write!(f, "Camera failed to open"),
            Error::OtherError(e) => write!(f, "{:?}", e),
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(t: anyhow::Error) -> Error {
        Self::OtherError(t)
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Error {
        Self::InvalidJSONConfgiFile
    }
}

impl From<IoError> for Error {
    fn from(_: IoError) -> Error {
        Self::OnnxModelFileNotFound
    }
}

impl From<NokhwaError> for Error {
    fn from(_: NokhwaError) -> Error {
        Self::CameraFailed
    }
}

impl std::error::Error for Error {}
