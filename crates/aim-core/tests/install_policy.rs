use aim_core::integration::policy::{IntegrationMode, resolve_install_policy};
use aim_core::platform::{DistroFamily, HostCapabilities, InstallScope};
use std::path::Path;

#[test]
fn immutable_system_request_downgrades_to_user_when_allowed() {
    let capabilities = HostCapabilities::immutable_user_only();
    let policy =
        resolve_install_policy(DistroFamily::Immutable, InstallScope::System, &capabilities)
            .unwrap();

    assert_eq!(policy.scope, InstallScope::User);
    assert_eq!(policy.integration_mode, IntegrationMode::Degraded);
    assert!(!policy.warnings.is_empty());
}

#[test]
fn nix_system_request_is_denied() {
    let error = resolve_install_policy(
        DistroFamily::Nix,
        InstallScope::System,
        &HostCapabilities::default(),
    )
    .unwrap_err();

    assert!(error.contains("not supported on Nix hosts"));
}

#[test]
fn system_policy_uses_managed_payload_and_native_integration_roots() {
    let policy = resolve_install_policy(
        DistroFamily::Fedora,
        InstallScope::System,
        &HostCapabilities::default(),
    )
    .unwrap();

    assert_eq!(policy.scope, InstallScope::System);
    assert_eq!(policy.payload_root, Path::new("/opt/aim/appimages"));
    assert_eq!(
        policy.desktop_entry_root,
        Path::new("/usr/share/applications")
    );
    assert_eq!(
        policy.icon_root,
        Path::new("/usr/share/icons/hicolor/256x256/apps")
    );
    assert_eq!(policy.integration_mode, IntegrationMode::Full);
}
