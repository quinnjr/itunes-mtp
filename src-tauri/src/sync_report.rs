use serde::Serialize;
use crate::errors::SyncError;

/// Detailed error information for a single operation
#[derive(Debug, Clone, Serialize)]
pub struct OperationError {
    pub operation: String,
    pub error: String,
    pub category: String,
    pub is_retryable: bool,
    pub attempts: u32,
    pub file_path: Option<String>,
    pub track_id: Option<String>,
}

impl From<(String, SyncError, u32)> for OperationError {
    fn from((operation, error, attempts): (String, SyncError, u32)) -> Self {
        Self {
            operation,
            error: error.message(),
            category: format!("{:?}", error.category()),
            is_retryable: error.is_retryable(),
            attempts,
            file_path: None,
            track_id: None,
        }
    }
}

/// Detailed sync report with comprehensive error information
#[derive(Debug, Clone, Serialize)]
pub struct SyncReport {
    pub success: bool,
    pub total_operations: u32,
    pub successful_operations: u32,
    pub failed_operations: u32,
    pub skipped_operations: u32,
    pub errors: Vec<OperationError>,
    pub warnings: Vec<String>,
    pub duration_ms: u64,
    pub message: String,
}

impl SyncReport {
    pub fn new() -> Self {
        Self {
            success: true,
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            skipped_operations: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
            duration_ms: 0,
            message: String::new(),
        }
    }

    pub fn add_error(&mut self, operation: String, error: SyncError, attempts: u32) {
        self.errors.push((operation, error, attempts).into());
        self.failed_operations += 1;
        self.success = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn increment_success(&mut self) {
        self.successful_operations += 1;
    }

    pub fn increment_skipped(&mut self) {
        self.skipped_operations += 1;
    }

    pub fn finalize(&mut self) {
        let total = self.successful_operations + self.failed_operations + self.skipped_operations;
        self.total_operations = total;

        if self.failed_operations == 0 && self.skipped_operations == 0 {
            self.message = format!("Successfully synced {} operation(s)", self.successful_operations);
        } else if self.failed_operations == 0 {
            self.message = format!(
                "Synced {} operation(s), skipped {} operation(s)",
                self.successful_operations,
                self.skipped_operations
            );
        } else {
            self.message = format!(
                "Synced {} operation(s), failed {} operation(s), skipped {} operation(s)",
                self.successful_operations,
                self.failed_operations,
                self.skipped_operations
            );
        }
    }

    pub fn errors_by_category(&self) -> std::collections::HashMap<String, usize> {
        let mut categories: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for error in &self.errors {
            *categories.entry(error.category.clone()).or_insert(0) += 1;
        }
        categories
    }
}

impl Default for SyncReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::SyncError;

    #[test]
    fn test_sync_report_new() {
        let report = SyncReport::new();
        assert!(report.success);
        assert_eq!(report.total_operations, 0);
        assert_eq!(report.errors.len(), 0);
    }

    #[test]
    fn test_sync_report_add_error() {
        let mut report = SyncReport::new();
        report.add_error(
            "upload_file".to_string(),
            SyncError::NetworkError("Connection lost".to_string()),
            3,
        );

        assert!(!report.success);
        assert_eq!(report.failed_operations, 1);
        assert_eq!(report.errors.len(), 1);
    }

    #[test]
    fn test_sync_report_finalize() {
        let mut report = SyncReport::new();
        report.increment_success();
        report.increment_success();
        report.add_error(
            "upload_file".to_string(),
            SyncError::FileNotFound("file.mp3".to_string()),
            1,
        );
        report.finalize();

        assert_eq!(report.total_operations, 3);
        assert_eq!(report.successful_operations, 2);
        assert_eq!(report.failed_operations, 1);
        assert!(!report.message.is_empty());
    }
}

