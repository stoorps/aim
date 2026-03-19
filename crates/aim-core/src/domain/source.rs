#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum SourceKind {
    GitHub,
    GitLab,
    DirectUrl,
    File,
}

impl SourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::GitLab => "gitlab",
            Self::DirectUrl => "direct-url",
            Self::File => "file",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum SourceInputKind {
    RepoShorthand,
    GitHubRepositoryUrl,
    GitHubReleaseUrl,
    GitHubReleaseAssetUrl,
    GitLabUrl,
    DirectUrl,
    File,
}

impl SourceInputKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RepoShorthand => "repo-shorthand",
            Self::GitHubRepositoryUrl => "github-repository-url",
            Self::GitHubReleaseUrl => "github-release-url",
            Self::GitHubReleaseAssetUrl => "github-release-asset-url",
            Self::GitLabUrl => "gitlab-url",
            Self::DirectUrl => "direct-url",
            Self::File => "file",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum NormalizedSourceKind {
    GitHubRepository,
    GitHubRelease,
    GitHubReleaseAsset,
    GitLab,
    DirectUrl,
    File,
}

impl NormalizedSourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GitHubRepository => "github-repository",
            Self::GitHubRelease => "github-release",
            Self::GitHubReleaseAsset => "github-release-asset",
            Self::GitLab => "gitlab",
            Self::DirectUrl => "direct-url",
            Self::File => "file",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SourceRef {
    pub kind: SourceKind,
    pub locator: String,
    #[serde(default = "default_source_input_kind")]
    pub input_kind: SourceInputKind,
    #[serde(default = "default_normalized_source_kind")]
    pub normalized_kind: NormalizedSourceKind,
    #[serde(default)]
    pub canonical_locator: Option<String>,
    #[serde(default)]
    pub requested_tag: Option<String>,
    #[serde(default)]
    pub requested_asset_name: Option<String>,
    #[serde(default)]
    pub tracks_latest: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ResolvedRelease {
    pub version: String,
    #[serde(default)]
    pub prerelease: bool,
}

const fn default_source_input_kind() -> SourceInputKind {
    SourceInputKind::DirectUrl
}

const fn default_normalized_source_kind() -> NormalizedSourceKind {
    NormalizedSourceKind::DirectUrl
}
