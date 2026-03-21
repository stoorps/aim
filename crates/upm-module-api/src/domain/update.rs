#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum UpdateChannelKind {
    GitHubReleases,
    ElectronBuilder,
    Zsync,
    DirectAsset,
}

impl UpdateChannelKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GitHubReleases => "github-releases",
            Self::ElectronBuilder => "electron-builder",
            Self::Zsync => "zsync",
            Self::DirectAsset => "direct-asset-lineage",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ChannelPreference {
    pub kind: UpdateChannelKind,
    pub locator: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct UpdateStrategy {
    pub preferred: ChannelPreference,
    #[serde(default)]
    pub alternates: Vec<ChannelPreference>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactCandidate {
    pub url: String,
    pub version: String,
    pub arch: Option<String>,
    pub trusted_checksum: Option<String>,
    pub weak_checksum_md5: Option<String>,
    pub selection_reason: String,
}
