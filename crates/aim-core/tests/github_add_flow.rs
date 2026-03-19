use aim_core::app::add::build_add_plan;
use aim_core::app::query::resolve_query;

#[test]
fn github_adapter_can_normalize_owner_repo_source() {
    let source = resolve_query("sharkdp/bat").unwrap();

    assert_eq!(source.kind.as_str(), "github");
}

#[test]
fn add_flow_builds_github_plan_from_owner_repo_query() {
    let plan = build_add_plan("sharkdp/bat").unwrap();

    assert_eq!(plan.resolution.source.kind.as_str(), "github");
    assert_eq!(plan.resolution.source.locator, "sharkdp/bat");
    assert_eq!(plan.resolution.release.version, "latest");
}
