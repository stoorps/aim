use aim_core::adapters::github::GitHubAdapter;
use aim_core::adapters::gitlab::GitLabAdapter;
use aim_core::adapters::traits::{AdapterCapabilities, SourceAdapter};

#[test]
fn adapter_capabilities_can_report_exact_resolution_only() {
    let capabilities = AdapterCapabilities::exact_resolution_only();
    assert!(!capabilities.supports_search);
}

#[test]
fn legacy_github_adapter_delegates_to_source_pipeline() {
    let adapter: &dyn SourceAdapter = &GitHubAdapter;

    let result = adapter.normalize("sharkdp/bat").unwrap();

    assert_eq!(result.normalized_kind.as_str(), "github-repository");
    assert_eq!(result.canonical_locator.as_deref(), Some("sharkdp/bat"));

    let resolution = adapter.resolve(&result).unwrap();
    assert_eq!(resolution.release.version, "latest");
}

#[test]
fn gitlab_adapter_normalizes_and_resolves_through_trait() {
    let adapter: &dyn SourceAdapter = &GitLabAdapter;

    let result = adapter
        .normalize("https://gitlab.com/example/team/app")
        .unwrap();

    assert_eq!(result.kind.as_str(), "gitlab");

    let resolution = adapter.resolve(&result).unwrap();
    assert_eq!(resolution.release.version, "latest");
}
