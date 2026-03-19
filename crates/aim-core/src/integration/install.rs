use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::{error::Error, fmt};

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
    pub staging_root: &'a Path,
    pub final_payload_path: &'a Path,
    pub artifact_bytes: &'a [u8],
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
    staging_root: &Path,
    final_payload_path: &Path,
    artifact_bytes: &[u8],
) -> Result<PayloadInstallOutcome, PayloadInstallError> {
    if !is_appimage_payload(artifact_bytes) {
        return Err(PayloadInstallError::InvalidArtifact);
    }

    let app_id = final_payload_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("download");
    let staged_path = staged_appimage_path(staging_root, app_id);
    let replacement = replacement_path(final_payload_path);

    fs::create_dir_all(staging_root)?;
    fs::write(&staged_path, artifact_bytes)?;

    let mut permissions = fs::metadata(&staged_path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&staged_path, permissions)?;

    if let Some(parent) = final_payload_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::rename(&staged_path, &replacement)?;
    fs::rename(&replacement, final_payload_path)?;

    Ok(PayloadInstallOutcome {
        final_payload_path: final_payload_path.to_path_buf(),
    })
}

fn is_appimage_payload(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\x7fELF")
}

pub fn execute_install(
    request: &InstallRequest<'_>,
) -> Result<InstallOutcome, PayloadInstallError> {
    let payload = stage_and_commit_payload(
        request.staging_root,
        request.final_payload_path,
        request.artifact_bytes,
    )?;

    let mut desktop_entry_path = None;
    let mut icon_path = None;
    if let Some(desktop) = &request.desktop {
        let extracted_icon = if desktop.icon_bytes.is_none() && desktop.icon_path.is_some() {
            extract_icon_from_payload(request.artifact_bytes)
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
