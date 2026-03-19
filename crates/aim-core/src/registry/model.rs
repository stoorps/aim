#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Registry {
    pub version: u32,
    pub apps: Vec<crate::domain::app::AppRecord>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            version: 1,
            apps: Vec::new(),
        }
    }
}
