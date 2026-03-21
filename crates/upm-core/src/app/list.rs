use crate::domain::app::AppRecord;

#[derive(Debug, Eq, PartialEq)]
pub struct ListRow {
    pub stable_id: String,
    pub display_name: String,
    pub version: Option<String>,
    pub source: String,
}

pub fn build_list_rows(apps: &[AppRecord]) -> Vec<ListRow> {
    apps.iter()
        .map(|app| ListRow {
            stable_id: app.stable_id.clone(),
            display_name: app.display_name.clone(),
            version: app.installed_version.clone(),
            source: app
                .source
                .as_ref()
                .map(|source| source.locator.clone())
                .or_else(|| app.source_input.clone())
                .unwrap_or_else(|| "-".to_owned()),
        })
        .collect()
}
