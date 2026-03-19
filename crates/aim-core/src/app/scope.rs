use crate::domain::app::InstallScope;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeOverride {
    System,
    User,
}

pub fn resolve_install_scope(
    _is_effective_root: bool,
    override_scope: ScopeOverride,
) -> InstallScope {
    match override_scope {
        ScopeOverride::System => InstallScope::System,
        ScopeOverride::User => InstallScope::User,
    }
}

pub fn resolve_install_scope_with_default(
    is_effective_root: bool,
    override_scope: Option<ScopeOverride>,
) -> InstallScope {
    match override_scope {
        Some(scope) => resolve_install_scope(is_effective_root, scope),
        None if is_effective_root => InstallScope::System,
        None => InstallScope::User,
    }
}
