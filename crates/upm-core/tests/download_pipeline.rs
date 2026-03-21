use std::fs;
use std::io::{self, Cursor, Read};
use std::time::Duration;

use tempfile::tempdir;
use upm_core::app::add::{
    InstallAppError, download_to_staged_path_with_retries,
    stream_payload_to_staged_file_with_reporter,
};
use upm_core::app::progress::{NoopReporter, OperationEvent};
use upm_core::integration::install::{InstallRequest, execute_install};
use upm_core::platform::DesktopHelpers;
use upm_core::source::github::HttpClientPolicy;

#[test]
fn payload_streaming_writes_staged_file_and_reports_progress() {
    let root = tempdir().unwrap();
    let staged_path = root.path().join("staging/bat.download");
    let bytes = b"\x7fELFAppImage";
    let mut reader = Cursor::new(bytes.as_slice());
    let mut events = Vec::new();
    let mut reporter = |event: &OperationEvent| events.push(event.clone());

    let written = stream_payload_to_staged_file_with_reporter(
        &mut reader,
        Some(bytes.len() as u64),
        &staged_path,
        &mut reporter,
    )
    .unwrap();

    assert_eq!(written, bytes.len() as u64);
    assert_eq!(
        fs::metadata(&staged_path).unwrap().len(),
        bytes.len() as u64
    );
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OperationEvent::Progress {
                current,
                total: Some(total)
            } if *current == bytes.len() as u64 && *total == bytes.len() as u64
        )
    }));
}

#[test]
fn install_commits_from_staged_payload_path() {
    let root = tempdir().unwrap();
    let staged_path = root.path().join("staging/bat.download");
    let final_payload_path = root.path().join("payloads/bat.AppImage");
    fs::create_dir_all(staged_path.parent().unwrap()).unwrap();
    fs::write(&staged_path, b"\x7fELFAppImage").unwrap();

    let outcome = execute_install(&InstallRequest {
        staged_payload_path: &staged_path,
        final_payload_path: &final_payload_path,
        trusted_checksum: None,
        weak_checksum_md5: None,
        desktop: None,
        helpers: DesktopHelpers::default(),
    })
    .unwrap();

    assert_eq!(outcome.final_payload_path, final_payload_path);
    assert!(outcome.final_payload_path.exists());
    assert!(!staged_path.exists());
}

#[test]
fn failed_streaming_download_removes_partial_staged_payload() {
    let root = tempdir().unwrap();
    let staged_path = root.path().join("staging/bat.download");
    let mut reader = FailingReader::new(b"\x7fELFpartial".to_vec(), 4);
    let mut reporter = NoopReporter;

    let result = stream_payload_to_staged_file_with_reporter(
        &mut reader,
        Some(12),
        &staged_path,
        &mut reporter,
    );

    assert!(result.is_err());
    assert!(!staged_path.exists());
}

#[test]
fn retry_policy_retries_transient_failures_before_success() {
    let root = tempdir().unwrap();
    let staged_path = root.path().join("staging/bat.download");
    let bytes = b"\x7fELFAppImage";
    let mut attempts = 0;

    let written = download_to_staged_path_with_retries(
        &staged_path,
        &mut NoopReporter,
        HttpClientPolicy {
            timeout: Duration::from_secs(30),
            max_retries: 3,
        },
        || {
            attempts += 1;
            if attempts == 1 {
                return Err(InstallAppError::DownloadIo(io::Error::other(
                    "transient failure",
                )));
            }

            Ok((
                Box::new(Cursor::new(bytes.to_vec())) as Box<dyn Read>,
                Some(bytes.len() as u64),
            ))
        },
    )
    .unwrap();

    assert_eq!(attempts, 2);
    assert_eq!(written, bytes.len() as u64);
    assert!(staged_path.exists());
}

#[test]
fn retry_exhaustion_returns_error_and_cleans_staged_payload() {
    let root = tempdir().unwrap();
    let staged_path = root.path().join("staging/bat.download");
    let mut attempts = 0;

    let result = download_to_staged_path_with_retries(
        &staged_path,
        &mut NoopReporter,
        HttpClientPolicy {
            timeout: Duration::from_secs(30),
            max_retries: 2,
        },
        || {
            attempts += 1;
            Ok((
                Box::new(FailingReader::new(b"\x7fELFpartial".to_vec(), 4)) as Box<dyn Read>,
                Some(12),
            ))
        },
    );

    assert!(result.is_err());
    assert_eq!(attempts, 2);
    assert!(!staged_path.exists());
}

struct FailingReader {
    bytes: Vec<u8>,
    chunk_size: usize,
    position: usize,
}

impl FailingReader {
    fn new(bytes: Vec<u8>, chunk_size: usize) -> Self {
        Self {
            bytes,
            chunk_size,
            position: 0,
        }
    }
}

impl Read for FailingReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.position >= self.chunk_size {
            return Err(io::Error::other("fixture read failure"));
        }

        let remaining = self.chunk_size - self.position;
        let to_read = remaining
            .min(buffer.len())
            .min(self.bytes.len() - self.position);
        buffer[..to_read].copy_from_slice(&self.bytes[self.position..self.position + to_read]);
        self.position += to_read;
        Ok(to_read)
    }
}
