use crate::domain::app::AppRecord;
pub use upm_module_api::domain::update::{
    ArtifactCandidate, ChannelPreference, UpdateChannelKind, UpdateStrategy,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum ParsedMetadataKind {
    Unknown,
    ElectronBuilder,
    Zsync,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct MetadataHints {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub primary_download: Option<String>,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(default)]
    pub architecture: Option<String>,
    #[serde(default)]
    pub channel_label: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ParsedMetadata {
    pub kind: ParsedMetadataKind,
    pub hints: MetadataHints,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub confidence: u8,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct UpdateChannel {
    pub kind: UpdateChannelKind,
    pub locator: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub artifact_name: Option<String>,
    #[serde(default)]
    pub confidence: u8,
    #[serde(default)]
    pub matches_install_origin: bool,
    #[serde(default)]
    pub prerelease: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdatePlan {
    pub items: Vec<PlannedUpdate>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlannedUpdate {
    pub stable_id: String,
    pub display_name: String,
    pub selected_channel: ChannelPreference,
    pub selection_reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateExecutionResult {
    pub apps: Vec<AppRecord>,
    pub items: Vec<ExecutedUpdate>,
}

impl UpdateExecutionResult {
    pub fn updated_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| item.status == UpdateExecutionStatus::Updated)
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| matches!(item.status, UpdateExecutionStatus::Failed { .. }))
            .count()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutedUpdate {
    pub stable_id: String,
    pub display_name: String,
    pub from_version: Option<String>,
    pub to_version: Option<String>,
    pub warnings: Vec<String>,
    pub status: UpdateExecutionStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpdateExecutionStatus {
    Updated,
    Failed { reason: String },
}
