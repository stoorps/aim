use crate::domain::app::InstallScope;
use crate::domain::source::SourceKind;
use crate::domain::update::{ParsedMetadataKind, UpdateChannelKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShowResult {
    Installed(InstalledShow),
    Remote(RemoteShow),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledShow {
    pub stable_id: String,
    pub display_name: String,
    pub installed_version: Option<String>,
    pub source_input: Option<String>,
    pub source: Option<SourceSummary>,
    pub install_scope: Option<InstallScope>,
    pub tracked_paths: TrackedInstallPaths,
    pub update_strategy: Option<UpdateStrategySummary>,
    pub metadata: Vec<MetadataSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteShow {
    pub source: SourceSummary,
    pub artifact: RemoteArtifactSummary,
    pub interactions: Vec<RemoteInteractionSummary>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceSummary {
    pub kind: SourceKind,
    pub locator: String,
    pub canonical_locator: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackedInstallPaths {
    pub payload_path: Option<String>,
    pub desktop_entry_path: Option<String>,
    pub icon_path: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateStrategySummary {
    pub preferred: UpdateChannelSummary,
    pub alternates: Vec<UpdateChannelSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateChannelSummary {
    pub kind: UpdateChannelKind,
    pub locator: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataSummary {
    pub kind: ParsedMetadataKind,
    pub version: Option<String>,
    pub primary_download: Option<String>,
    pub checksum: Option<String>,
    pub architecture: Option<String>,
    pub channel_label: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteArtifactSummary {
    pub url: String,
    pub version: Option<String>,
    pub arch: Option<String>,
    pub trusted_checksum: Option<String>,
    pub selection_reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RemoteInteractionSummary {
    ChooseTrackingPreference {
        requested_version: String,
        latest_version: String,
    },
    SelectArtifact {
        candidate_count: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShowResultError {
    AmbiguousInstalledMatch {
        query: String,
        matches: Vec<String>,
    },
    UnsupportedQuery,
    InsecureHttpSource,
    NoInstallableArtifact {
        source: SourceSummary,
    },
    AdapterResolutionFailed {
        adapter_id: String,
        kind: AdapterFailureKind,
        detail: Option<String>,
    },
    GitHubDiscoveryFailed {
        kind: GitHubDiscoveryFailureKind,
        detail: Option<String>,
    },
    NoInstallableCandidates,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterFailureKind {
    UnsupportedQuery,
    UnsupportedSource,
    ResolutionFailed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GitHubDiscoveryFailureKind {
    Unsupported,
    FixtureDocumentMissing,
    NoReleases,
    Transport,
}
