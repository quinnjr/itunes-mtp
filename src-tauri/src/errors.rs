// These error types/variants are part of the sync error taxonomy; several are
// only constructed by the Windows-only sync command (or kept for completeness),
// so allow dead code for the whole module on every target.
#![allow(dead_code)]

use quick_xml::Error as XmlError;
use serde::Serialize;
use std::{io, string::FromUtf8Error};

#[derive(Debug, Clone, Serialize)]
pub enum SyncError {
    IoError(String),
    XmlError(String),
    Utf8Error(String),
    ParseError(String),
    TransferError(String),
    DeviceError(String),
    FileNotFound(String),
    CorruptedFile(String),
    NetworkError(String),
    TimeoutError(String),
    Custom(String),
}

/// Error category for error reporting
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub enum ErrorCategory {
    FileSystem,    // File I/O errors (missing files, permission issues)
    Network,       // Network/connection errors
    Device,        // Device-specific errors (disconnected, incompatible)
    Corruption,    // File corruption or invalid format
    Timeout,       // Operation timeout
    Configuration, // Configuration/parameter errors
    Unknown,       // Unclassified errors
}

impl SyncError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            SyncError::IoError(_) | SyncError::FileNotFound(_) => ErrorCategory::FileSystem,
            SyncError::NetworkError(_) => ErrorCategory::Network,
            SyncError::DeviceError(_) => ErrorCategory::Device,
            SyncError::CorruptedFile(_) => ErrorCategory::Corruption,
            SyncError::TimeoutError(_) => ErrorCategory::Timeout,
            SyncError::ParseError(_) | SyncError::Utf8Error(_) => ErrorCategory::Configuration,
            _ => ErrorCategory::Unknown,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self.category(),
            ErrorCategory::Network | ErrorCategory::Device | ErrorCategory::Timeout
        )
    }

    pub fn message(&self) -> String {
        match self {
            SyncError::IoError(msg) => format!("IO error: {}", msg),
            SyncError::XmlError(msg) => format!("XML error: {}", msg),
            SyncError::Utf8Error(msg) => format!("UTF-8 error: {}", msg),
            SyncError::ParseError(msg) => format!("Parse error: {}", msg),
            SyncError::TransferError(msg) => format!("Transfer error: {}", msg),
            SyncError::DeviceError(msg) => format!("Device error: {}", msg),
            SyncError::FileNotFound(msg) => format!("File not found: {}", msg),
            SyncError::CorruptedFile(msg) => format!("Corrupted file: {}", msg),
            SyncError::NetworkError(msg) => format!("Network error: {}", msg),
            SyncError::TimeoutError(msg) => format!("Timeout error: {}", msg),
            SyncError::Custom(msg) => msg.clone(),
        }
    }
}

impl From<io::Error> for SyncError {
    fn from(error: io::Error) -> Self {
        SyncError::IoError(error.to_string())
    }
}

impl From<XmlError> for SyncError {
    fn from(error: XmlError) -> Self {
        SyncError::XmlError(error.to_string())
    }
}

impl From<FromUtf8Error> for SyncError {
    fn from(error: FromUtf8Error) -> Self {
        SyncError::Utf8Error(error.to_string())
    }
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for SyncError {}
