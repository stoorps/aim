use crate::domain::source::SourceRef;
use crate::domain::update::{ParsedMetadata, UpdateStrategy};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstallScope {
    User,
    System,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityConfidence {
    Confident,
    NeedsConfirmation,
    RawUrlFallback,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AppIdentity {
    pub stable_id: String,
    pub display_name: String,
    pub confidence: IdentityConfidence,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct AppRecord {
    pub stable_id: String,
    pub display_name: String,
    #[serde(default)]
    pub source_input: Option<String>,
    #[serde(default)]
    pub source: Option<SourceRef>,
    #[serde(default)]
    pub installed_version: Option<String>,
    #[serde(default)]
    pub update_strategy: Option<UpdateStrategy>,
    #[serde(default)]
    pub metadata: Vec<ParsedMetadata>,
}
