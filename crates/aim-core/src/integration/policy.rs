use std::path::PathBuf;

use crate::domain::app::InstallScope;
use crate::platform::{
    DistroFamily, HostCapabilities, system_applications_dir, system_icons_dir,
    system_managed_appimages_dir,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntegrationMode {
    Full,
    Degraded,
    PayloadOnly,
    Denied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallPolicy {
    pub scope: InstallScope,
    pub payload_root: PathBuf,
    pub desktop_entry_root: PathBuf,
    pub icon_root: PathBuf,
    pub integration_mode: IntegrationMode,
    pub warnings: Vec<String>,
}

pub fn resolve_install_policy(
    family: DistroFamily,
    requested_scope: InstallScope,
    capabilities: &HostCapabilities,
) -> Result<InstallPolicy, String> {
    match (family, requested_scope) {
        (DistroFamily::Nix, InstallScope::System) => Err(
            "system installs are not supported on Nix hosts until a native strategy exists"
                .to_string(),
        ),
        (DistroFamily::Immutable, InstallScope::System) if capabilities.is_immutable => {
            Ok(InstallPolicy {
                scope: InstallScope::User,
                payload_root: PathBuf::from(".local/lib/aim/appimages"),
                desktop_entry_root: PathBuf::from(".local/share/applications"),
                icon_root: PathBuf::from(".local/share/icons/hicolor/256x256/apps"),
                integration_mode: IntegrationMode::Degraded,
                warnings: vec![
                    "system install requested on immutable host; downgraded to user scope"
                        .to_string(),
                ],
            })
        }
        (_, InstallScope::System) => Ok(InstallPolicy {
            scope: InstallScope::System,
            payload_root: system_managed_appimages_dir(),
            desktop_entry_root: system_applications_dir(),
            icon_root: system_icons_dir(),
            integration_mode: IntegrationMode::Full,
            warnings: Vec::new(),
        }),
        _ => Ok(InstallPolicy {
            scope: InstallScope::User,
            payload_root: PathBuf::from(".local/lib/aim/appimages"),
            desktop_entry_root: PathBuf::from(".local/share/applications"),
            icon_root: PathBuf::from(".local/share/icons/hicolor/256x256/apps"),
            integration_mode: if capabilities.has_desktop_session {
                IntegrationMode::Full
            } else {
                IntegrationMode::PayloadOnly
            },
            warnings: Vec::new(),
        }),
    }
}
