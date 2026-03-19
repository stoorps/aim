use aim_core::app::query::resolve_query;
use aim_core::domain::source::SourceKind;

#[test]
fn owner_repo_defaults_to_github() {
    let source = resolve_query("sharkdp/bat").unwrap();
    assert_eq!(source.kind, SourceKind::GitHub);
}
