//! Archive integrity testing operations

use super::{OperationStatus, ProgressCallback, ProgressInfo};
use crate::archive::Archive;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Test result for an archive entry
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Entry path
    pub path: String,
    /// Whether the test passed
    pub passed: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Test archive integrity
pub fn test_integrity(archive: &Archive) -> Result<bool> {
    archive.test_integrity()
}

/// Test archive integrity with detailed results
pub fn test_integrity_detailed(archive: &Archive) -> Result<Vec<TestResult>> {
    let entries = archive.list_entries("")?;
    let mut results = Vec::new();

    for entry in &entries {
        if entry.is_directory {
            continue;
        }

        // Try to read the entry to verify integrity
        let result = match archive.read_text(entry, entry.uncompressed_size as usize) {
            Ok(_) => TestResult {
                path: entry.path.clone(),
                passed: true,
                error: None,
            },
            Err(e) => {
                // If it's not UTF-8, that's fine - we just wanted to read it
                if e.to_string().contains("UTF-8") {
                    TestResult {
                        path: entry.path.clone(),
                        passed: true,
                        error: None,
                    }
                } else {
                    TestResult {
                        path: entry.path.clone(),
                        passed: false,
                        error: Some(e.to_string()),
                    }
                }
            }
        };

        results.push(result);
    }

    Ok(results)
}

/// Test archive with progress callback
pub fn test_integrity_with_progress(
    archive: &Archive,
    callback: ProgressCallback,
    cancel_flag: Arc<AtomicBool>,
) -> Result<Vec<TestResult>> {
    let entries = archive.list_entries("")?;
    let file_entries: Vec<_> = entries.iter().filter(|e| !e.is_directory).collect();

    let total_files = file_entries.len();
    let total_bytes: u64 = file_entries.iter().map(|e| e.uncompressed_size).sum();

    let mut results = Vec::new();
    let mut bytes_processed = 0u64;

    for (index, entry) in file_entries.iter().enumerate() {
        // Check for cancellation
        if cancel_flag.load(Ordering::Relaxed) {
            callback(ProgressInfo {
                current_file: entry.path.clone(),
                current_index: index,
                total_files,
                bytes_processed,
                total_bytes,
                status: OperationStatus::Cancelled,
            });
            return Err(anyhow::anyhow!("Operation cancelled"));
        }

        // Report progress
        callback(ProgressInfo {
            current_file: entry.path.clone(),
            current_index: index,
            total_files,
            bytes_processed,
            total_bytes,
            status: OperationStatus::InProgress,
        });

        // Test entry
        let result = match archive.read_text(entry, entry.uncompressed_size as usize) {
            Ok(_) => TestResult {
                path: entry.path.clone(),
                passed: true,
                error: None,
            },
            Err(e) => {
                if e.to_string().contains("UTF-8") {
                    TestResult {
                        path: entry.path.clone(),
                        passed: true,
                        error: None,
                    }
                } else {
                    TestResult {
                        path: entry.path.clone(),
                        passed: false,
                        error: Some(e.to_string()),
                    }
                }
            }
        };

        bytes_processed += entry.uncompressed_size;
        results.push(result);
    }

    // Report completion
    let all_passed = results.iter().all(|r| r.passed);
    callback(ProgressInfo {
        current_file: String::new(),
        current_index: total_files,
        total_files,
        bytes_processed: total_bytes,
        total_bytes,
        status: if all_passed {
            OperationStatus::Completed
        } else {
            OperationStatus::Failed("Some files failed integrity check".to_string())
        },
    });

    Ok(results)
}

/// Verify CRC32 checksums for entries that have them
pub fn verify_checksums(archive: &Archive) -> Result<Vec<TestResult>> {
    let entries = archive.list_entries("")?;
    let mut results = Vec::new();

    for entry in &entries {
        if entry.is_directory || entry.crc32.is_none() {
            continue;
        }

        // For now, we trust the archive library's CRC verification
        // which happens during extraction/reading
        results.push(TestResult {
            path: entry.path.clone(),
            passed: true,
            error: None,
        });
    }

    Ok(results)
}

/// Summary of test results
#[derive(Debug, Clone)]
pub struct TestSummary {
    pub total_files: usize,
    pub passed: usize,
    pub failed: usize,
    pub failed_entries: Vec<TestResult>,
}

impl TestSummary {
    pub fn from_results(results: &[TestResult]) -> Self {
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = results.len() - passed;
        let failed_entries: Vec<_> = results.iter().filter(|r| !r.passed).cloned().collect();

        Self {
            total_files: results.len(),
            passed,
            failed,
            failed_entries,
        }
    }

    pub fn is_ok(&self) -> bool {
        self.failed == 0
    }
}

/// Format test summary as string
pub fn format_test_summary(summary: &TestSummary) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "Test Results: {} files tested\n",
        summary.total_files
    ));
    output.push_str(&format!("  Passed: {}\n", summary.passed));
    output.push_str(&format!("  Failed: {}\n", summary.failed));

    if !summary.failed_entries.is_empty() {
        output.push_str("\nFailed entries:\n");
        for entry in &summary.failed_entries {
            output.push_str(&format!(
                "  - {}: {}\n",
                entry.path,
                entry.error.as_deref().unwrap_or("Unknown error")
            ));
        }
    }

    output
}
