use std::{io, string::FromUtf8Error};
use quick_xml::Error as XmlError;

#[derive(Debug)]
pub enum SyncError {
    IoError(io::Error),
    XmlError(XmlError),
    Utf8Error(FromUtf8Error),
    ParseError(String),  // New variant for parse errors
    Custom(String),
}

impl From<io::Error> for SyncError {
    fn from(error: io::Error) -> Self {
        SyncError::IoError(error)
    }
}

impl From<XmlError> for SyncError {
    fn from(error: XmlError) -> Self {
        SyncError::XmlError(error)
    }
}

impl From<FromUtf8Error> for SyncError {
    fn from(error: FromUtf8Error) -> Self {
        SyncError::Utf8Error(error)
    }
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::IoError(e) => write!(f, "IO error: {}", e),
            SyncError::XmlError(e) => write!(f, "XML error: {}", e),
            SyncError::Utf8Error(e) => write!(f, "UTF-8 error: {}", e),
            SyncError::ParseError(s) => write!(f, "Parse error: {}", s),  // New match arm for parse errors
            SyncError::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for SyncError {}