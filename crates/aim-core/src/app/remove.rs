use crate::app::interaction::{InteractionKind, InteractionRequest};
use crate::domain::app::AppRecord;

pub fn resolve_registered_app<'a>(
    query: &str,
    apps: &'a [AppRecord],
) -> Result<&'a AppRecord, ResolveRegisteredAppError> {
    let normalized_query = normalize_lookup(query);
    let matches = apps
        .iter()
        .filter(|app| {
            normalize_lookup(&app.stable_id) == normalized_query
                || normalize_lookup(&app.display_name) == normalized_query
        })
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [] => Err(ResolveRegisteredAppError::UnknownApp {
            query: query.to_owned(),
        }),
        [app] => Ok(*app),
        _ => Err(ResolveRegisteredAppError::Ambiguous {
            request: InteractionRequest {
                key: "select-registered-app".to_owned(),
                kind: InteractionKind::SelectRegisteredApp {
                    query: query.to_owned(),
                    matches: matches
                        .iter()
                        .map(|app| format!("{} ({})", app.display_name, app.stable_id))
                        .collect(),
                },
            },
        }),
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct RemovalPlan {
    pub stable_id: String,
    pub display_name: String,
    pub artifact_paths: Vec<String>,
}

pub fn build_removal_plan(app: &AppRecord) -> RemovalPlan {
    RemovalPlan {
        stable_id: app.stable_id.clone(),
        display_name: app.display_name.clone(),
        artifact_paths: Vec::new(),
    }
}

pub fn remove_registered_app(
    query: &str,
    apps: &[AppRecord],
) -> Result<RemovalResult, ResolveRegisteredAppError> {
    let app = resolve_registered_app(query, apps)?;
    let remaining_apps = apps
        .iter()
        .filter(|candidate| candidate.stable_id != app.stable_id)
        .cloned()
        .collect();

    Ok(RemovalResult {
        removed: build_removal_plan(app),
        remaining_apps,
    })
}

#[derive(Debug, Eq, PartialEq)]
pub struct RemovalResult {
    pub removed: RemovalPlan,
    pub remaining_apps: Vec<AppRecord>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ResolveRegisteredAppError {
    UnknownApp { query: String },
    Ambiguous { request: InteractionRequest },
}

fn normalize_lookup(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}
