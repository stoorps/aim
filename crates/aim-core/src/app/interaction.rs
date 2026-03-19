#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InteractionRequest {
    pub key: String,
    pub kind: InteractionKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InteractionKind {
    SelectRegisteredApp {
        query: String,
        matches: Vec<String>,
    },
    ChooseTrackingPreference {
        requested_version: String,
        latest_version: String,
    },
    SelectArtifact {
        candidates: Vec<String>,
    },
}
