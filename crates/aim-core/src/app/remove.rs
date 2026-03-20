use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::app::interaction::{InteractionKind, InteractionRequest};
use crate::app::progress::{
    NoopReporter, OperationEvent, OperationKind, OperationStage, ProgressReporter,
};
use crate::domain::app::{AppRecord, InstallScope};
use crate::integration::paths::{desktop_entry_path, icon_path, managed_appimage_path};
use crate::integration::refresh::refresh_integration;
use crate::platform::probe_live_host;

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

pub fn build_removal_plan(app: &AppRecord, install_home: &Path) -> RemovalPlan {
    let artifact_paths = removal_artifact_paths(app, install_home)
        .into_iter()
        .map(|path| path.display().to_string())
        .collect();

    RemovalPlan {
        stable_id: app.stable_id.clone(),
        display_name: app.display_name.clone(),
        artifact_paths,
    }
}

pub fn remove_registered_app(
    query: &str,
    apps: &[AppRecord],
    install_home: &Path,
) -> Result<RemovalResult, RemoveRegisteredAppError> {
    let mut reporter = NoopReporter;
    remove_registered_app_with_reporter(query, apps, install_home, &mut reporter)
}

pub fn remove_registered_app_with_reporter(
    query: &str,
    apps: &[AppRecord],
    install_home: &Path,
    reporter: &mut impl ProgressReporter,
) -> Result<RemovalResult, RemoveRegisteredAppError> {
    reporter.report(&OperationEvent::Started {
        kind: OperationKind::Remove,
        label: query.to_owned(),
    });
    reporter.report(&OperationEvent::StageChanged {
        stage: OperationStage::ResolveQuery,
        message: format!("resolving {query}"),
    });
    let app = resolve_registered_app(query, apps).map_err(RemoveRegisteredAppError::Resolve)?;
    let plan = build_removal_plan(app, install_home);
    reporter.report(&OperationEvent::StageChanged {
        stage: OperationStage::StagePayload,
        message: "removing managed artifacts".to_owned(),
    });
    let warnings = delete_artifacts(&plan)?;
    let remaining_apps = apps
        .iter()
        .filter(|candidate| candidate.stable_id != app.stable_id)
        .cloned()
        .collect();

    let result = RemovalResult {
        removed: plan,
        remaining_apps,
        warnings,
    };

    reporter.report(&OperationEvent::StageChanged {
        stage: OperationStage::Finalize,
        message: format!("removed {}", result.removed.stable_id),
    });
    reporter.report(&OperationEvent::Finished {
        summary: format!("removed {}", result.removed.stable_id),
    });

    Ok(result)
}

#[derive(Debug, Eq, PartialEq)]
pub struct RemovalResult {
    pub removed: RemovalPlan,
    pub remaining_apps: Vec<AppRecord>,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub enum RemoveRegisteredAppError {
    Resolve(ResolveRegisteredAppError),
    Io(io::Error),
}

#[derive(Debug, Eq, PartialEq)]
pub enum ResolveRegisteredAppError {
    UnknownApp { query: String },
    Ambiguous { request: InteractionRequest },
}

fn normalize_lookup(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn removal_artifact_paths(app: &AppRecord, install_home: &Path) -> Vec<PathBuf> {
    if let Some(install) = &app.install {
        return [
            install.payload_path.as_deref(),
            install.desktop_entry_path.as_deref(),
            install.icon_path.as_deref(),
        ]
        .into_iter()
        .flatten()
        .map(PathBuf::from)
        .collect();
    }

    let scope = InstallScope::User;
    vec![
        managed_appimage_path(install_home, scope, &app.stable_id),
        desktop_entry_path(install_home, scope, &app.stable_id),
        icon_path(install_home, scope, &app.stable_id),
    ]
}

fn delete_artifacts(plan: &RemovalPlan) -> Result<Vec<String>, RemoveRegisteredAppError> {
    let desktop_path = plan.artifact_paths.get(1).map(PathBuf::from);
    let icon_path = plan.artifact_paths.get(2).map(PathBuf::from);

    for artifact_path in &plan.artifact_paths {
        match fs::remove_file(artifact_path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(RemoveRegisteredAppError::Io(error)),
        }
    }

    let mut warnings = Vec::new();
    if let Ok((_, capabilities)) = probe_live_host(Path::new("/"), InstallScope::User) {
        warnings.extend(refresh_integration(
            &capabilities.helpers,
            desktop_path.as_deref(),
            icon_path.as_deref(),
        ));
    }

    Ok(warnings)
}
