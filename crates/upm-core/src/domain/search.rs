pub const DEFAULT_REMOTE_LIMIT: usize = 10;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchInstallStatus {
    Available,
    Installed {
        installed_version: Option<String>,
    },
    UpdateAvailable {
        installed_version: Option<String>,
        latest_version: Option<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchQuery {
    pub text: String,
    pub remote_limit: usize,
}

impl SearchQuery {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            remote_limit: DEFAULT_REMOTE_LIMIT,
        }
    }

    pub fn with_remote_limit(text: &str, remote_limit: usize) -> Self {
        Self {
            text: text.to_owned(),
            remote_limit,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchResult {
    pub provider_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub source_locator: String,
    pub install_query: String,
    pub canonical_locator: String,
    pub version: Option<String>,
    pub install_status: SearchInstallStatus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstalledSearchMatch {
    pub stable_id: String,
    pub display_name: String,
    pub installed_version: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchWarning {
    pub provider_id: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchResults {
    pub query_text: String,
    pub remote_hits: Vec<SearchResult>,
    pub installed_matches: Vec<InstalledSearchMatch>,
    pub warnings: Vec<SearchWarning>,
}
