use aim_core::adapters::github::GitHubAdapter;
use aim_core::adapters::traits::AdapterCapabilities;

#[test]
fn adapter_capabilities_can_report_exact_resolution_only() {
    let capabilities = AdapterCapabilities::exact_resolution_only();
    assert!(!capabilities.supports_search);
}

#[test]
fn legacy_github_adapter_delegates_to_source_pipeline() {
    let adapter = GitHubAdapter;

    let result = adapter.normalize("sharkdp/bat").unwrap();

    assert_eq!(result.normalized_kind.as_str(), "github-repository");
    assert_eq!(result.canonical_locator.as_deref(), Some("sharkdp/bat"));
}
