use upm_core::adapters::traits::AdapterResolution;
use upm_core::app::UpmApp;
use upm_core::app::add::AddSecurityPolicy;
use upm_core::app::providers::{ExternalAddProvider, ExternalAddResolution, ProviderRegistry};
use upm_core::app::search::{SearchProvider, SearchProviderError};
use upm_core::domain::search::{SearchInstallStatus, SearchQuery, SearchResult};
use upm_core::domain::source::{
    NormalizedSourceKind, ResolvedRelease, SourceInputKind, SourceKind, SourceRef,
};
use upm_core::domain::update::{
    ArtifactCandidate, ChannelPreference, UpdateChannelKind, UpdateStrategy,
};
use upm_core::source::github::FixtureGitHubTransport;

struct StubSearchProvider;

impl SearchProvider for StubSearchProvider {
    fn search(&self, _query: &SearchQuery) -> Result<Vec<SearchResult>, SearchProviderError> {
        Ok(vec![SearchResult {
            provider_id: "external-search".to_owned(),
            display_name: "Firefox Nightly".to_owned(),
            description: Some("Provided by facade-owned providers".to_owned()),
            source_locator: "https://example.invalid/firefox-nightly".to_owned(),
            install_query: "external/firefox-nightly".to_owned(),
            canonical_locator: "external/firefox-nightly".to_owned(),
            version: Some("2026.03.21".to_owned()),
            install_status: SearchInstallStatus::Available,
        }])
    }
}

struct StubExternalAddProvider;

impl ExternalAddProvider for StubExternalAddProvider {
    fn id(&self) -> &'static str {
        "stub-appimage"
    }

    fn resolve(
        &self,
        source: &SourceRef,
    ) -> Result<Option<ExternalAddResolution>, upm_core::adapters::traits::AdapterError> {
        Ok(
            (source.kind == SourceKind::AppImageHub).then(|| ExternalAddResolution {
                resolution: AdapterResolution {
                    source: SourceRef {
                        kind: SourceKind::AppImageHub,
                        locator: source.locator.clone(),
                        input_kind: SourceInputKind::AppImageHubShorthand,
                        normalized_kind: NormalizedSourceKind::AppImageHub,
                        canonical_locator: Some("2338455".to_owned()),
                        requested_tag: None,
                        requested_asset_name: None,
                        tracks_latest: true,
                    },
                    release: ResolvedRelease {
                        version: "stable".to_owned(),
                        prerelease: false,
                    },
                },
                selected_artifact: ArtifactCandidate {
                    url: "https://downloads.example.invalid/firefox.AppImage".to_owned(),
                    version: "stable".to_owned(),
                    arch: Some("x86_64".to_owned()),
                    trusted_checksum: None,
                    weak_checksum_md5: Some("deadbeef".to_owned()),
                    selection_reason: "provider-release".to_owned(),
                },
                update_strategy: UpdateStrategy {
                    preferred: ChannelPreference {
                        kind: UpdateChannelKind::DirectAsset,
                        locator: "https://downloads.example.invalid/firefox.AppImage".to_owned(),
                        reason: "provider-release".to_owned(),
                    },
                    alternates: Vec::new(),
                },
                display_name_hint: Some(
                    "Firefox by Mozilla - Official AppImage Edition".to_owned(),
                ),
            }),
        )
    }
}

#[test]
fn upm_app_can_be_constructed_without_cli_owned_module_composition() {
    let _app = UpmApp::new();
}

#[test]
fn upm_app_search_delegates_through_the_application_facade() {
    let app = UpmApp::builder()
        .with_github_transport(Box::new(FixtureGitHubTransport))
        .with_provider_registry(
            ProviderRegistry::default().with_search_provider(StubSearchProvider),
        )
        .build();

    let results = app.search(&SearchQuery::new("firefox"), &[]).unwrap();

    assert!(results.remote_hits.iter().any(|hit| {
        hit.provider_id == "external-search" && hit.install_query == "external/firefox-nightly"
    }));
}

#[test]
fn upm_app_add_planning_delegates_through_the_application_facade() {
    let app = UpmApp::builder()
        .with_github_transport(Box::new(FixtureGitHubTransport))
        .with_provider_registry(
            ProviderRegistry::default().with_external_add_provider(StubExternalAddProvider),
        )
        .build();

    let plan = app
        .build_add_plan("appimagehub/2338455", AddSecurityPolicy::default())
        .unwrap();

    assert_eq!(plan.resolution.source.kind, SourceKind::AppImageHub);
    assert_eq!(
        plan.selected_artifact.url,
        "https://downloads.example.invalid/firefox.AppImage"
    );
}
