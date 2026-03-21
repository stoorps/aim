use upm_core::app::add::{AddSecurityPolicy, build_add_plan_with_registered_providers};
use upm_core::app::providers::{ExternalAddProvider, ExternalAddResolution, ProviderRegistry};
use upm_core::app::search::{SearchProvider, build_search_results_with_registered_providers};
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
    fn search(
        &self,
        _query: &SearchQuery,
    ) -> Result<Vec<SearchResult>, upm_core::app::search::SearchProviderError> {
        Ok(vec![SearchResult {
            provider_id: "external-search".to_owned(),
            display_name: "Firefox Nightly".to_owned(),
            description: Some("Provided by external registry".to_owned()),
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
                resolution: upm_core::adapters::traits::AdapterResolution {
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
fn build_search_results_with_registered_providers_uses_external_hits() {
    let query = SearchQuery::new("firefox");
    let search_provider = StubSearchProvider;
    let providers = ProviderRegistry {
        search_providers: vec![&search_provider],
        external_add_providers: Vec::new(),
    };

    let results = build_search_results_with_registered_providers(&query, &[], &providers).unwrap();

    let external_hit = results
        .remote_hits
        .iter()
        .find(|hit| hit.provider_id == "external-search")
        .unwrap();

    assert_eq!(external_hit.install_query, "external/firefox-nightly");
    assert!(
        results
            .remote_hits
            .iter()
            .all(|hit| hit.provider_id != "appimagehub")
    );
}

#[test]
fn build_add_plan_with_registered_providers_requires_external_provider_for_appimagehub() {
    let registry = ProviderRegistry::default();

    let error = build_add_plan_with_registered_providers(
        "appimagehub/2338455",
        &FixtureGitHubTransport,
        &registry,
        AddSecurityPolicy::default(),
    )
    .unwrap_err();

    assert!(matches!(
        error,
        upm_core::app::add::BuildAddPlanError::NoInstallableArtifact { .. }
    ));
}

#[test]
fn build_add_plan_with_registered_providers_delegates_appimagehub_like_sources() {
    let provider = StubExternalAddProvider;
    let registry = ProviderRegistry {
        search_providers: Vec::new(),
        external_add_providers: vec![&provider],
    };

    let plan = build_add_plan_with_registered_providers(
        "appimagehub/2338455",
        &FixtureGitHubTransport,
        &registry,
        AddSecurityPolicy::default(),
    )
    .unwrap();

    assert_eq!(plan.resolution.source.kind, SourceKind::AppImageHub);
    assert_eq!(
        plan.resolution.source.canonical_locator.as_deref(),
        Some("2338455")
    );
    assert_eq!(
        plan.selected_artifact.url,
        "https://downloads.example.invalid/firefox.AppImage"
    );
    assert_eq!(
        plan.display_name_hint.as_deref(),
        Some("Firefox by Mozilla - Official AppImage Edition")
    );
}
