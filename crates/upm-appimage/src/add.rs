use crate::source::appimagehub::{
    AppImageHubError, AppImageHubTransport, resolve_appimagehub_item, resolve_appimagehub_item_with,
};
use upm_module_api::adapters::traits::{
    AdapterCapabilities, AdapterError, AdapterResolution, AdapterResolveOutcome, SourceAdapter,
};
use upm_module_api::app::providers::{ExternalAddProvider, ExternalAddResolution};
use upm_module_api::domain::source::{
    NormalizedSourceKind, ResolvedRelease, SourceInputKind, SourceKind, SourceRef,
};
use upm_module_api::domain::update::{
    ArtifactCandidate, ChannelPreference, UpdateChannelKind, UpdateStrategy,
};

pub struct AppImageHubAdapter;

impl AppImageHubAdapter {
    pub fn resolve_source_with<T: AppImageHubTransport + ?Sized>(
        &self,
        source: &SourceRef,
        transport: &T,
    ) -> Result<AdapterResolveOutcome, AdapterError> {
        if source.kind != SourceKind::AppImageHub {
            return Err(AdapterError::UnsupportedSource);
        }

        let resolved = resolve_appimagehub_item_with(source, transport)
            .map_err(|error| AdapterError::ResolutionFailed(render_appimagehub_error(&error)))?;

        match resolved {
            Some(item) => Ok(AdapterResolveOutcome::Resolved(AdapterResolution {
                source: item.source,
                release: ResolvedRelease {
                    version: item.version,
                    prerelease: false,
                },
            })),
            None => Ok(AdapterResolveOutcome::NoInstallableArtifact {
                source: source.clone(),
            }),
        }
    }
}

impl SourceAdapter for AppImageHubAdapter {
    fn id(&self) -> &'static str {
        "appimagehub"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            supports_search: true,
            supports_exact_resolution: true,
        }
    }

    fn repository_source_kind(&self) -> Option<SourceKind> {
        Some(SourceKind::AppImageHub)
    }

    fn normalize(&self, query: &str) -> Result<SourceRef, AdapterError> {
        let source = resolve_appimagehub_query(query)?;
        if source.kind != SourceKind::AppImageHub {
            return Err(AdapterError::UnsupportedQuery);
        }

        Ok(source)
    }

    fn resolve(&self, source: &SourceRef) -> Result<AdapterResolution, AdapterError> {
        match resolve_appimagehub_item(source)
            .map_err(|error| AdapterError::ResolutionFailed(render_appimagehub_error(&error)))?
        {
            Some(item) => Ok(AdapterResolution {
                source: item.source,
                release: ResolvedRelease {
                    version: item.version,
                    prerelease: false,
                },
            }),
            None => Err(AdapterError::ResolutionFailed(
                "appimagehub item has no installable AppImage artifact".to_owned(),
            )),
        }
    }

    fn resolve_supported_source(
        &self,
        source: &SourceRef,
    ) -> Result<AdapterResolveOutcome, AdapterError> {
        let transport = crate::source::appimagehub::default_transport();
        self.resolve_source_with(source, transport.as_ref())
    }
}

pub struct AppImageHubAddProvider {
    transport: Box<dyn AppImageHubTransport>,
}

impl AppImageHubAddProvider {
    pub fn new(transport: Box<dyn AppImageHubTransport>) -> Self {
        Self { transport }
    }
}

impl ExternalAddProvider for AppImageHubAddProvider {
    fn id(&self) -> &'static str {
        "appimagehub"
    }

    fn resolve(&self, source: &SourceRef) -> Result<Option<ExternalAddResolution>, AdapterError> {
        if source.kind != SourceKind::AppImageHub {
            return Ok(None);
        }

        let adapter = AppImageHubAdapter;
        let resolution = match adapter.resolve_source_with(source, self.transport.as_ref())? {
            AdapterResolveOutcome::Resolved(resolution) => resolution,
            AdapterResolveOutcome::NoInstallableArtifact { .. } => return Ok(None),
        };
        let Some(resolved_item) = resolve_appimagehub_item_with(source, self.transport.as_ref())
            .map_err(|error| AdapterError::ResolutionFailed(format!("{error:?}")))?
        else {
            return Ok(None);
        };

        Ok(Some(ExternalAddResolution {
            resolution,
            selected_artifact: ArtifactCandidate {
                url: resolved_item.download.url.clone(),
                version: resolved_item.version.clone(),
                arch: resolved_item.download.arch.clone(),
                trusted_checksum: None,
                weak_checksum_md5: resolved_item.download.md5sum.clone(),
                selection_reason: "provider-release".to_owned(),
            },
            update_strategy: UpdateStrategy {
                preferred: ChannelPreference {
                    kind: UpdateChannelKind::DirectAsset,
                    locator: resolved_item.download.url.clone(),
                    reason: "provider-release".to_owned(),
                },
                alternates: Vec::new(),
            },
            display_name_hint: Some(resolved_item.title),
        }))
    }
}

fn render_appimagehub_error(error: &AppImageHubError) -> String {
    match error {
        AppImageHubError::FixtureItemMissing(id) => {
            format!("missing appimagehub fixture item {id}")
        }
        AppImageHubError::InsecureDownloadUrl(url) => {
            format!("insecure appimagehub download url: {url}")
        }
        AppImageHubError::Parse(error) => error.to_string(),
        AppImageHubError::Transport(error) => error.to_string(),
        AppImageHubError::UnsupportedSource(locator) => {
            format!("unsupported appimagehub source: {locator}")
        }
    }
}

fn resolve_appimagehub_query(query: &str) -> Result<SourceRef, AdapterError> {
    let trimmed = query.trim();
    let id = if let Some(id) = trimmed.strip_prefix("appimagehub/") {
        id
    } else if let Some(id) = trimmed.strip_prefix("https://www.appimagehub.com/p/") {
        id
    } else if let Some(id) = trimmed.strip_prefix("http://www.appimagehub.com/p/") {
        id
    } else {
        return Err(AdapterError::UnsupportedQuery);
    };

    if !id.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(AdapterError::UnsupportedQuery);
    }

    Ok(SourceRef {
        kind: SourceKind::AppImageHub,
        locator: format!("https://www.appimagehub.com/p/{id}"),
        input_kind: if trimmed.starts_with("appimagehub/") {
            SourceInputKind::AppImageHubShorthand
        } else {
            SourceInputKind::AppImageHubUrl
        },
        normalized_kind: NormalizedSourceKind::AppImageHub,
        canonical_locator: Some(id.to_owned()),
        requested_tag: None,
        requested_asset_name: None,
        tracks_latest: true,
    })
}
