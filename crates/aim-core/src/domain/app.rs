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
}
