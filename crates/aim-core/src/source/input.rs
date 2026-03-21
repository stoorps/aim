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

    if let Some(classified) = classify_gitlab_http(query) {
        return classified;
    }

    if let Some(classified) = classify_sourceforge_http(query) {
        return classified;
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

fn classify_gitlab_http(query: &str) -> Option<Result<ClassifiedInput, ClassifyInputError>> {
    let trimmed = query
        .trim_start_matches("https://gitlab.com/")
        .trim_start_matches("http://gitlab.com/");
    if trimmed == query {
        return None;
    }

    let trimmed = trim_query_and_fragment(trimmed);

    let parts = trimmed
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    if parts.len() < 2 {
        return Some(Err(ClassifyInputError::Unsupported));
    }

    let release_marker = parts.iter().position(|segment| *segment == "-");
    let is_repository_url = release_marker.is_none() && is_supported_gitlab_repo_path(&parts);
    let is_release_like_url = matches!(release_marker, Some(index) if index >= 2)
        && parts.get(release_marker.unwrap() + 1) == Some(&"releases")
        && parts.get(release_marker.unwrap() + 2).is_some()
        && parts.len() == release_marker.unwrap() + 3;
    let is_ambiguous_candidate =
        release_marker.is_none() && is_ambiguous_gitlab_candidate_path(&parts);
    if !is_repository_url && !is_release_like_url && !is_ambiguous_candidate {
        return Some(Err(ClassifyInputError::Unsupported));
    }

    let canonical_parts = if let Some(index) = release_marker {
        &parts[..index]
    } else {
        &parts[..]
    };
    let canonical_locator = canonical_parts.join("/");
    let requested_tag = if let Some(index) = release_marker {
        parts.get(index + 2).map(|value| (*value).to_owned())
    } else {
        None
    };
    let tracks_latest = requested_tag.is_none() && !is_ambiguous_candidate;

    Some(Ok(ClassifiedInput {
        kind: SourceInputKind::GitLabUrl,
        source_kind: SourceKind::GitLab,
        normalized_kind: if is_ambiguous_candidate {
            NormalizedSourceKind::GitLabCandidate
        } else {
            NormalizedSourceKind::GitLab
        },
        locator: query.to_owned(),
        canonical_locator: if is_ambiguous_candidate {
            None
        } else {
            Some(canonical_locator)
        },
        requested_tag,
        requested_asset_name: None,
        tracks_latest,
    }))
}

fn classify_sourceforge_http(query: &str) -> Option<Result<ClassifiedInput, ClassifyInputError>> {
    let trimmed = query
        .trim_start_matches("https://sourceforge.net/projects/")
        .trim_start_matches("http://sourceforge.net/projects/");
    if trimmed == query {
        return None;
    }

    let trimmed = trim_query_and_fragment(trimmed);

    let parts = trimmed
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    let Some(project) = parts.first() else {
        return Some(Err(ClassifyInputError::Unsupported));
    };

    let is_project_url = parts.len() == 1;
    let is_latest_download_url =
        parts.len() == 4 && parts[1] == "files" && parts[2] == "latest" && parts[3] == "download";
    let is_root_file_download_url = parts.len() == 4
        && parts[1] == "files"
        && parts[3] == "download"
        && !matches!(parts[2], "latest" | "releases");
    let is_nested_file_download_url = parts.len() > 4
        && parts[1] == "files"
        && parts.last() == Some(&"download")
        && parts
            .get(parts.len().saturating_sub(2))
            .is_some_and(|segment| segment.contains('.'));
    let is_ambiguous_candidate = is_ambiguous_sourceforge_candidate_path(&parts);
    let is_concrete_download_url =
        !is_latest_download_url && (is_root_file_download_url || is_nested_file_download_url);
    if is_concrete_download_url {
        return Some(Ok(ClassifiedInput {
            kind: SourceInputKind::DirectUrl,
            source_kind: SourceKind::DirectUrl,
            normalized_kind: NormalizedSourceKind::DirectUrl,
            locator: query.to_owned(),
            canonical_locator: None,
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: false,
        }));
    }
    if !is_project_url && !is_latest_download_url && !is_ambiguous_candidate {
        return Some(Err(ClassifyInputError::Unsupported));
    }

    Some(Ok(ClassifiedInput {
        kind: SourceInputKind::SourceForgeUrl,
        source_kind: SourceKind::SourceForge,
        normalized_kind: if is_ambiguous_candidate {
            NormalizedSourceKind::SourceForgeCandidate
        } else {
            NormalizedSourceKind::SourceForge
        },
        locator: query.to_owned(),
        canonical_locator: Some((*project).to_owned()),
        requested_tag: None,
        requested_asset_name: None,
        tracks_latest: is_project_url || is_latest_download_url,
    }))
}

fn trim_query_and_fragment(value: &str) -> &str {
    value.split(['?', '#']).next().unwrap_or(value)
}

fn is_supported_gitlab_repo_path(parts: &[&str]) -> bool {
    if parts.len() < 2 {
        return false;
    }

    if parts.len() == 2 {
        return true;
    }

    if parts.len() == 3 {
        return !is_reserved_gitlab_resource_segment(parts[2]);
    }

    if parts[2..]
        .iter()
        .copied()
        .any(is_reserved_gitlab_resource_segment)
    {
        return false;
    }

    true
}

fn is_reserved_gitlab_resource_segment(segment: &str) -> bool {
    matches!(
        segment,
        "issues"
            | "merge_requests"
            | "releases"
            | "tags"
            | "blob"
            | "tree"
            | "commits"
            | "packages"
            | "archive"
            | "raw"
            | "pipelines"
            | "jobs"
            | "wikis"
            | "snippets"
    )
}

fn is_ambiguous_gitlab_candidate_path(parts: &[&str]) -> bool {
    parts.len() == 4 && parts[2] == "releases"
}

fn is_ambiguous_sourceforge_candidate_path(parts: &[&str]) -> bool {
    parts.len() == 5
        && parts[1] == "files"
        && parts[2] == "releases"
        && (parts[3] == "stable" || is_version_like_sourceforge_folder(parts[3]))
        && parts[4] == "download"
}

fn is_version_like_sourceforge_folder(segment: &str) -> bool {
    segment.starts_with('v') && segment.chars().any(|character| character.is_ascii_digit())
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
