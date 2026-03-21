use crate::adapters::traits::{
    AdapterCapabilities, AdapterError, AdapterResolution, AdapterResolveOutcome, SourceAdapter,
};
use crate::app::query::resolve_query;
use crate::domain::source::{NormalizedSourceKind, ResolvedRelease, SourceKind, SourceRef};

pub struct SourceForgeAdapter;

impl SourceForgeAdapter {
    pub fn artifact_url(source: &SourceRef) -> Option<String> {
        if let Some(asset_name) = source.requested_asset_name.as_deref()
            && is_sourceforge_releases_root_locator(&source.locator)
        {
            return Some(format!("{}/{asset_name}/download", source.locator));
        }

        if is_latest_download_locator(&source.locator)
            || is_sourceforge_release_folder_download_locator(&source.locator)
        {
            return Some(source.locator.clone());
        }

        if is_sourceforge_releases_root_locator(&source.locator) {
            return sourceforge_latest_download_url(&source.locator);
        }

        None
    }
}

impl SourceAdapter for SourceForgeAdapter {
    fn id(&self) -> &'static str {
        "sourceforge"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            supports_search: true,
            supports_exact_resolution: true,
        }
    }

    fn repository_source_kind(&self) -> Option<SourceKind> {
        Some(SourceKind::SourceForge)
    }

    fn normalize(&self, query: &str) -> Result<SourceRef, AdapterError> {
        let source = resolve_query(query).map_err(|_| AdapterError::UnsupportedQuery)?;
        if source.kind != SourceKind::SourceForge {
            return Err(AdapterError::UnsupportedQuery);
        }

        Ok(source)
    }

    fn resolve(&self, source: &SourceRef) -> Result<AdapterResolution, AdapterError> {
        if source.kind != SourceKind::SourceForge {
            return Err(AdapterError::UnsupportedSource);
        }
        if !is_resolved_download_locator(&source.locator) {
            return Err(AdapterError::ResolutionFailed(
                "sourceforge source has no concrete latest-download artifact".to_owned(),
            ));
        }

        Ok(AdapterResolution {
            source: resolved_source(source),
            release: ResolvedRelease {
                version: "latest".to_owned(),
                prerelease: false,
            },
        })
    }

    fn resolve_supported_source(
        &self,
        source: &SourceRef,
    ) -> Result<AdapterResolveOutcome, AdapterError> {
        if Self::artifact_url(source).is_some() {
            return self.resolve(source).map(AdapterResolveOutcome::Resolved);
        }

        if matches!(
            source.normalized_kind,
            NormalizedSourceKind::SourceForge | NormalizedSourceKind::SourceForgeCandidate
        ) {
            return Ok(AdapterResolveOutcome::NoInstallableArtifact {
                source: source.clone(),
            });
        }

        Ok(AdapterResolveOutcome::NoInstallableArtifact {
            source: source.clone(),
        })
    }
}

fn resolved_source(source: &SourceRef) -> SourceRef {
    let mut resolved = source.clone();
    if is_sourceforge_file_like_release_download_locator(&resolved.locator) {
        resolved.locator = sourceforge_releases_root_url(&resolved.locator)
            .unwrap_or_else(|| resolved.locator.clone());
        resolved.normalized_kind = NormalizedSourceKind::SourceForge;
        resolved.tracks_latest = true;
    } else if is_sourceforge_release_folder_download_locator(&resolved.locator)
        || is_sourceforge_releases_root_locator(&resolved.locator)
    {
        resolved.normalized_kind = NormalizedSourceKind::SourceForge;
        resolved.tracks_latest = true;
    }

    resolved
}

fn is_resolved_download_locator(locator: &str) -> bool {
    is_latest_download_locator(locator)
        || is_sourceforge_release_folder_download_locator(locator)
        || is_sourceforge_releases_root_locator(locator)
}

fn is_latest_download_locator(locator: &str) -> bool {
    let trimmed = locator
        .split(['?', '#'])
        .next()
        .unwrap_or(locator)
        .trim_end_matches('/');
    trimmed.ends_with("/files/latest/download")
}

fn is_sourceforge_release_folder_download_locator(locator: &str) -> bool {
    let trimmed = locator
        .split(['?', '#'])
        .next()
        .unwrap_or(locator)
        .trim_end_matches('/');

    let parts = trimmed
        .trim_start_matches("https://sourceforge.net/projects/")
        .trim_start_matches("http://sourceforge.net/projects/")
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    parts.len() == 5 && parts[1] == "files" && parts[2] == "releases" && parts[4] == "download"
}

fn is_sourceforge_file_like_release_download_locator(locator: &str) -> bool {
    let trimmed = locator
        .split(['?', '#'])
        .next()
        .unwrap_or(locator)
        .trim_end_matches('/');

    let parts = trimmed
        .trim_start_matches("https://sourceforge.net/projects/")
        .trim_start_matches("http://sourceforge.net/projects/")
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    parts.len() == 5
        && parts[1] == "files"
        && parts[2] == "releases"
        && is_sourceforge_artifact_name(parts[3])
        && parts[4] == "download"
}

fn is_sourceforge_artifact_name(segment: &str) -> bool {
    let lower = segment.to_ascii_lowercase();

    [
        ".appimage",
        ".tar.gz",
        ".tar.xz",
        ".tar.bz2",
        ".zip",
        ".deb",
        ".rpm",
        ".exe",
        ".msi",
        ".dmg",
        ".pkg",
        ".apk",
        ".tgz",
        ".whl",
        ".jar",
        ".nupkg",
    ]
    .iter()
    .any(|suffix| lower.ends_with(suffix))
}

fn is_sourceforge_releases_root_locator(locator: &str) -> bool {
    let trimmed = locator
        .split(['?', '#'])
        .next()
        .unwrap_or(locator)
        .trim_end_matches('/');

    let parts = trimmed
        .trim_start_matches("https://sourceforge.net/projects/")
        .trim_start_matches("http://sourceforge.net/projects/")
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    parts.len() == 3 && parts[1] == "files" && parts[2] == "releases"
}

fn sourceforge_releases_root_url(locator: &str) -> Option<String> {
    let trimmed = locator
        .split(['?', '#'])
        .next()
        .unwrap_or(locator)
        .trim_end_matches('/');

    let prefix = if trimmed.starts_with("https://sourceforge.net/projects/") {
        "https://sourceforge.net/projects/"
    } else if trimmed.starts_with("http://sourceforge.net/projects/") {
        "http://sourceforge.net/projects/"
    } else {
        return None;
    };

    let path = trimmed
        .trim_start_matches("https://sourceforge.net/projects/")
        .trim_start_matches("http://sourceforge.net/projects/")
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    if path.is_empty() {
        return None;
    }

    Some(format!("{}{}/files/releases", prefix, path[0]))
}

fn sourceforge_latest_download_url(locator: &str) -> Option<String> {
    let trimmed = locator
        .split(['?', '#'])
        .next()
        .unwrap_or(locator)
        .trim_end_matches('/');

    let prefix = if trimmed.starts_with("https://sourceforge.net/projects/") {
        "https://sourceforge.net/projects/"
    } else if trimmed.starts_with("http://sourceforge.net/projects/") {
        "http://sourceforge.net/projects/"
    } else {
        return None;
    };

    let path = trimmed
        .trim_start_matches("https://sourceforge.net/projects/")
        .trim_start_matches("http://sourceforge.net/projects/")
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    if path.is_empty() {
        return None;
    }

    Some(format!("{}{}/files/latest/download", prefix, path[0]))
}
