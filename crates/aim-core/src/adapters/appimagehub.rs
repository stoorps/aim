use crate::adapters::traits::{
    AdapterCapabilities, AdapterError, AdapterResolution, AdapterResolveOutcome, SourceAdapter,
};
use crate::app::query::resolve_query;
use crate::domain::source::{ResolvedRelease, SourceKind, SourceRef};
use crate::source::appimagehub::{
    AppImageHubError, AppImageHubTransport, resolve_appimagehub_item, resolve_appimagehub_item_with,
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
        let source = resolve_query(query).map_err(|_| AdapterError::UnsupportedQuery)?;
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
