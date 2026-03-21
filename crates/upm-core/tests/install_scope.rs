use upm_core::app::scope::{ScopeOverride, resolve_install_scope};
use upm_core::domain::app::InstallScope;

#[test]
fn explicit_scope_override_beats_effective_user() {
    let scope = resolve_install_scope(false, ScopeOverride::System);
    assert_eq!(scope, InstallScope::System);
}
