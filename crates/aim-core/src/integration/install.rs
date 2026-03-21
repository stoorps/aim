use std::fs;
use std::io;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::{error::Error, fmt};

use base64::Engine;
use sha2::{Digest, Sha512};

use crate::integration::desktop::{extract_icon_from_payload, write_desktop_integration};
use crate::integration::refresh::refresh_integration;
use crate::platform::DesktopHelpers;

pub fn staged_appimage_path(staging_root: &Path, app_id: &str) -> PathBuf {
    staging_root.join(format!("{app_id}.download"))
}

pub fn replacement_path(target: &Path) -> PathBuf {
    let mut file_name = target
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_default();
    file_name.push(".new");
    target.with_file_name(file_name)
}

#[derive(Debug)]
pub enum PayloadInstallError {
    InvalidArtifact,
    ChecksumMismatch,
    InvalidTrustedChecksum,
    InvalidWeakChecksum,
    WeakChecksumMismatch,
    Io(io::Error),
    DesktopIntegration(io::Error),
}

impl From<io::Error> for PayloadInstallError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl fmt::Display for PayloadInstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArtifact => write!(f, "artifact is not a valid AppImage"),
            Self::ChecksumMismatch => write!(f, "artifact checksum did not match trusted metadata"),
            Self::InvalidTrustedChecksum => write!(f, "trusted checksum metadata is malformed"),
            Self::InvalidWeakChecksum => write!(f, "weak provider checksum metadata is malformed"),
            Self::WeakChecksumMismatch => {
                write!(
                    f,
                    "weak provider checksum did not match downloaded artifact"
                )
            }
            Self::Io(error) => write!(f, "payload installation failed: {error}"),
            Self::DesktopIntegration(error) => {
                write!(f, "desktop integration failed: {error}")
            }
        }
    }
}

impl Error for PayloadInstallError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayloadInstallOutcome {
    pub final_payload_path: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopIntegrationRequest<'a> {
    pub desktop_entry_path: &'a Path,
    pub desktop_entry_contents: &'a str,
    pub icon_path: Option<&'a Path>,
    pub icon_bytes: Option<&'a [u8]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallRequest<'a> {
    pub staged_payload_path: &'a Path,
    pub final_payload_path: &'a Path,
    pub trusted_checksum: Option<&'a str>,
    pub weak_checksum_md5: Option<&'a str>,
    pub desktop: Option<DesktopIntegrationRequest<'a>>,
    pub helpers: DesktopHelpers,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallOutcome {
    pub final_payload_path: PathBuf,
    pub desktop_entry_path: Option<PathBuf>,
    pub icon_path: Option<PathBuf>,
    pub warnings: Vec<String>,
}

pub fn stage_and_commit_payload(
    staged_payload_path: &Path,
    final_payload_path: &Path,
) -> Result<PayloadInstallOutcome, PayloadInstallError> {
    if !is_appimage_payload_path(staged_payload_path)? {
        let _ = fs::remove_file(staged_payload_path);
        return Err(PayloadInstallError::InvalidArtifact);
    }

    let replacement = replacement_path(final_payload_path);

    let mut permissions = fs::metadata(staged_payload_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(staged_payload_path, permissions)?;

    if let Some(parent) = final_payload_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::rename(staged_payload_path, &replacement)?;
    fs::rename(&replacement, final_payload_path)?;

    Ok(PayloadInstallOutcome {
        final_payload_path: final_payload_path.to_path_buf(),
    })
}

fn is_appimage_payload_path(path: &Path) -> Result<bool, io::Error> {
    let mut file = fs::File::open(path)?;
    let mut header = [0_u8; 4];
    let read = file.read(&mut header)?;
    Ok(read == header.len() && header == *b"\x7fELF")
}

pub fn execute_install(
    request: &InstallRequest<'_>,
) -> Result<InstallOutcome, PayloadInstallError> {
    verify_trusted_checksum(request.staged_payload_path, request.trusted_checksum)?;
    verify_weak_checksum_md5(request.staged_payload_path, request.weak_checksum_md5)?;
    let payload =
        stage_and_commit_payload(request.staged_payload_path, request.final_payload_path)?;

    let mut desktop_entry_path = None;
    let mut icon_path = None;
    if let Some(desktop) = &request.desktop {
        let extracted_icon = if desktop.icon_bytes.is_none() && desktop.icon_path.is_some() {
            extract_icon_from_payload_path(&payload.final_payload_path)
        } else {
            None
        };
        let written = write_desktop_integration(
            desktop.desktop_entry_path,
            desktop.desktop_entry_contents,
            desktop.icon_path,
            desktop.icon_bytes.or(extracted_icon.as_deref()),
        )
        .map_err(|error| {
            let _ = fs::remove_file(&payload.final_payload_path);
            PayloadInstallError::DesktopIntegration(error)
        })?;
        desktop_entry_path = Some(written.desktop_entry_path);
        icon_path = written.icon_path;
    }

    let warnings = refresh_integration(
        &request.helpers,
        desktop_entry_path.as_deref(),
        icon_path.as_deref(),
    );

    Ok(InstallOutcome {
        final_payload_path: payload.final_payload_path,
        desktop_entry_path,
        icon_path,
        warnings,
    })
}

fn extract_icon_from_payload_path(path: &Path) -> Option<Vec<u8>> {
    fs::read(path)
        .ok()
        .and_then(|payload| extract_icon_from_payload(&payload))
}

fn verify_trusted_checksum(
    staged_payload_path: &Path,
    trusted_checksum: Option<&str>,
) -> Result<(), PayloadInstallError> {
    let Some(trusted_checksum) = trusted_checksum.map(str::trim) else {
        return Ok(());
    };

    let decoded = base64::engine::general_purpose::STANDARD
        .decode(trusted_checksum)
        .map_err(|_| {
            let _ = fs::remove_file(staged_payload_path);
            PayloadInstallError::InvalidTrustedChecksum
        })?;
    if decoded.len() != 64 {
        let _ = fs::remove_file(staged_payload_path);
        return Err(PayloadInstallError::InvalidTrustedChecksum);
    }

    let payload = fs::read(staged_payload_path)?;
    let actual_checksum = base64::engine::general_purpose::STANDARD.encode(Sha512::digest(payload));
    if actual_checksum != trusted_checksum {
        let _ = fs::remove_file(staged_payload_path);
        return Err(PayloadInstallError::ChecksumMismatch);
    }

    Ok(())
}

fn verify_weak_checksum_md5(
    staged_payload_path: &Path,
    weak_checksum_md5: Option<&str>,
) -> Result<(), PayloadInstallError> {
    let Some(weak_checksum_md5) = weak_checksum_md5.map(str::trim) else {
        return Ok(());
    };

    if weak_checksum_md5.len() != 32
        || !weak_checksum_md5
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        let _ = fs::remove_file(staged_payload_path);
        return Err(PayloadInstallError::InvalidWeakChecksum);
    }

    let payload = fs::read(staged_payload_path)?;
    let actual_checksum = format!("{:x}", md5::compute(payload));
    if actual_checksum != weak_checksum_md5.to_ascii_lowercase() {
        let _ = fs::remove_file(staged_payload_path);
        return Err(PayloadInstallError::WeakChecksumMismatch);
    }

    Ok(())
}
