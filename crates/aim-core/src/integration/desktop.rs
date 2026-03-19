use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesktopWriteOutcome {
    pub desktop_entry_path: PathBuf,
    pub icon_path: Option<PathBuf>,
}

pub fn write_desktop_integration(
    desktop_entry_path: &Path,
    desktop_entry_contents: &str,
    icon_path: Option<&Path>,
    icon_bytes: Option<&[u8]>,
) -> Result<DesktopWriteOutcome, io::Error> {
    if let Some(parent) = desktop_entry_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(desktop_entry_path, desktop_entry_contents)?;

    let written_icon_path = match (icon_path, icon_bytes) {
        (Some(path), Some(bytes)) => {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, bytes)?;
            Some(path.to_path_buf())
        }
        _ => None,
    };

    Ok(DesktopWriteOutcome {
        desktop_entry_path: desktop_entry_path.to_path_buf(),
        icon_path: written_icon_path,
    })
}

pub fn extract_icon_from_payload(payload: &[u8]) -> Option<Vec<u8>> {
    const PNG_HEADER: &[u8] = b"\x89PNG\r\n\x1a\n";
    const PNG_TRAILER: &[u8] = b"IEND\xaeB`\x82";

    let start = payload
        .windows(PNG_HEADER.len())
        .position(|window| window == PNG_HEADER)?;
    let tail = payload[start..]
        .windows(PNG_TRAILER.len())
        .position(|window| window == PNG_TRAILER)?;
    let end = start + tail + PNG_TRAILER.len();

    Some(payload[start..end].to_vec())
}
