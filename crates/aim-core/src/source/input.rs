use crate::domain::source::{NormalizedSourceKind, SourceInputKind, SourceKind, SourceRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClassifiedInput {
    pub kind: SourceInputKind,
    pub source_kind: SourceKind,
    pub normalized_kind: NormalizedSourceKind,
    pub locator: String,
    pub canonical_locator: Option<String>,
    pub requested_tag: Option<String>,
    pub requested_asset_name: Option<String>,
    pub tracks_latest: bool,
}

impl ClassifiedInput {
    pub fn into_source_ref(self) -> SourceRef {
        SourceRef {
            kind: self.source_kind,
            locator: self.locator,
            input_kind: self.kind,
            normalized_kind: self.normalized_kind,
            canonical_locator: self.canonical_locator,
            requested_tag: self.requested_tag,
            requested_asset_name: self.requested_asset_name,
            tracks_latest: self.tracks_latest,
        }
    }
}

pub fn classify_input(query: &str) -> Result<ClassifiedInput, ClassifyInputError> {
    if query.starts_with("file://") {
        return Ok(ClassifiedInput {
            kind: SourceInputKind::File,
            source_kind: SourceKind::File,
            normalized_kind: NormalizedSourceKind::File,
            locator: query.to_owned(),
            canonical_locator: None,
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: false,
        });
    }

    if let Some(classified) = classify_github_http(query) {
        return Ok(classified);
    }

    if query.starts_with("https://gitlab.com/") || query.starts_with("http://gitlab.com/") {
        return Ok(ClassifiedInput {
            kind: SourceInputKind::GitLabUrl,
            source_kind: SourceKind::GitLab,
            normalized_kind: NormalizedSourceKind::GitLab,
            locator: query.to_owned(),
            canonical_locator: None,
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: false,
        });
    }

    if query.starts_with("https://") || query.starts_with("http://") {
        return Ok(ClassifiedInput {
            kind: SourceInputKind::DirectUrl,
            source_kind: SourceKind::DirectUrl,
            normalized_kind: NormalizedSourceKind::DirectUrl,
            locator: query.to_owned(),
            canonical_locator: None,
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: false,
        });
    }

    if is_github_shorthand(query) {
        return Ok(ClassifiedInput {
            kind: SourceInputKind::RepoShorthand,
            source_kind: SourceKind::GitHub,
            normalized_kind: NormalizedSourceKind::GitHubRepository,
            locator: query.to_owned(),
            canonical_locator: Some(query.to_owned()),
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: true,
        });
    }

    Err(ClassifyInputError::Unsupported)
}

#[derive(Debug, Eq, PartialEq)]
pub enum ClassifyInputError {
    Unsupported,
}

fn classify_github_http(query: &str) -> Option<ClassifiedInput> {
    let trimmed = query
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/");
    if trimmed == query {
        return None;
    }

    let parts = trimmed
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    if parts.len() < 2 {
        return None;
    }

    let repo = format!("{}/{}", parts[0], parts[1]);

    if parts.len() >= 5 && parts[2] == "releases" && parts[3] == "tag" {
        return Some(ClassifiedInput {
            kind: SourceInputKind::GitHubReleaseUrl,
            source_kind: SourceKind::GitHub,
            normalized_kind: NormalizedSourceKind::GitHubRelease,
            locator: query.to_owned(),
            canonical_locator: Some(repo),
            requested_tag: Some(parts[4].to_owned()),
            requested_asset_name: None,
            tracks_latest: false,
        });
    }

    if parts.len() >= 6 && parts[2] == "releases" && parts[3] == "download" {
        return Some(ClassifiedInput {
            kind: SourceInputKind::GitHubReleaseAssetUrl,
            source_kind: SourceKind::GitHub,
            normalized_kind: NormalizedSourceKind::GitHubReleaseAsset,
            locator: query.to_owned(),
            canonical_locator: Some(repo),
            requested_tag: Some(parts[4].to_owned()),
            requested_asset_name: Some(parts[5].to_owned()),
            tracks_latest: false,
        });
    }

    Some(ClassifiedInput {
        kind: SourceInputKind::GitHubRepositoryUrl,
        source_kind: SourceKind::GitHub,
        normalized_kind: NormalizedSourceKind::GitHubRepository,
        locator: query.to_owned(),
        canonical_locator: Some(repo),
        requested_tag: None,
        requested_asset_name: None,
        tracks_latest: true,
    })
}

fn is_github_shorthand(query: &str) -> bool {
    let mut parts = query.split('/');
    let Some(owner) = parts.next() else {
        return false;
    };
    let Some(repo) = parts.next() else {
        return false;
    };

    if parts.next().is_some() {
        return false;
    }

    !owner.is_empty() && !repo.is_empty() && !owner.contains(':') && !repo.contains(':')
}
