use upm_core::app::identity::{IdentityFallback, resolve_identity};
use upm_core::domain::app::IdentityConfidence;

#[test]
fn unresolved_identity_can_fall_back_to_url() {
    let identity = resolve_identity(
        None,
        None,
        Some("https://example.com/app.AppImage"),
        IdentityFallback::AllowRawUrl,
    )
    .unwrap();

    assert!(identity.stable_id.contains("example.com"));
    assert_eq!(identity.confidence, IdentityConfidence::RawUrlFallback);
}

#[test]
fn explicit_id_is_treated_as_confident() {
    let identity = resolve_identity(
        Some("Bat"),
        Some("sharkdp/bat"),
        Some("https://github.com/sharkdp/bat/releases"),
        IdentityFallback::AllowRawUrl,
    )
    .unwrap();

    assert_eq!(identity.stable_id, "sharkdp-bat");
    assert_eq!(identity.display_name, "Bat");
    assert_eq!(identity.confidence, IdentityConfidence::Confident);
}

#[test]
fn identifiers_containing_dot_dot_are_rejected() {
    let error = resolve_identity(
        Some("Bat"),
        Some(".."),
        Some("https://example.com/app.AppImage"),
        IdentityFallback::AllowRawUrl,
    )
    .unwrap_err();

    assert_eq!(
        error,
        upm_core::app::identity::ResolveIdentityError::InvalidStableId
    );
}
