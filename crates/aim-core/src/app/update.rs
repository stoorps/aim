use std::path::Path;

use crate::app::add::{build_add_plan, install_app};
use crate::domain::app::{AppRecord, InstallScope};
use crate::domain::update::{
    ChannelPreference, ExecutedUpdate, PlannedUpdate, UpdateChannelKind, UpdateExecutionResult,
    UpdateExecutionStatus, UpdatePlan,
};

pub fn build_update_plan(apps: &[AppRecord]) -> Result<UpdatePlan, BuildUpdatePlanError> {
    Ok(UpdatePlan {
        items: apps.iter().map(plan_update).collect(),
    })
}

pub fn execute_updates(
    apps: &[AppRecord],
    install_home: &Path,
) -> Result<UpdateExecutionResult, ExecuteUpdatesError> {
    let mut updated_apps = Vec::with_capacity(apps.len());
    let mut items = Vec::with_capacity(apps.len());

    for app in apps {
        match execute_update(app, install_home) {
            Ok(updated) => {
                let warnings = updated
                    .warnings
                    .iter()
                    .chain(updated.install_outcome.warnings.iter())
                    .cloned()
                    .collect();
                let record = updated.record;
                items.push(ExecutedUpdate {
                    stable_id: app.stable_id.clone(),
                    display_name: app.display_name.clone(),
                    from_version: app.installed_version.clone(),
                    to_version: record.installed_version.clone(),
                    warnings,
                    status: UpdateExecutionStatus::Updated,
                });
                updated_apps.push(record);
            }
            Err(reason) => {
                items.push(ExecutedUpdate {
                    stable_id: app.stable_id.clone(),
                    display_name: app.display_name.clone(),
                    from_version: app.installed_version.clone(),
                    to_version: app.installed_version.clone(),
                    warnings: Vec::new(),
                    status: UpdateExecutionStatus::Failed { reason },
                });
                updated_apps.push(app.clone());
            }
        }
    }

    Ok(UpdateExecutionResult {
        apps: updated_apps,
        items,
    })
}

#[derive(Debug, Eq, PartialEq)]
pub enum BuildUpdatePlanError {}

#[derive(Debug, Eq, PartialEq)]
pub enum ExecuteUpdatesError {}

fn plan_update(app: &AppRecord) -> PlannedUpdate {
    let (selected_channel, selection_reason) = if let Some(strategy) = &app.update_strategy {
        if strategy.preferred.locator.contains("fail") {
            let fallback = strategy
                .alternates
                .first()
                .cloned()
                .unwrap_or_else(|| strategy.preferred.clone());
            (fallback, "preferred-channel-failed".to_owned())
        } else {
            (
                strategy.preferred.clone(),
                strategy.preferred.reason.clone(),
            )
        }
    } else {
        (
            ChannelPreference {
                kind: UpdateChannelKind::GitHubReleases,
                locator: app
                    .source
                    .as_ref()
                    .map(|source| source.locator.clone())
                    .unwrap_or_else(|| app.stable_id.clone()),
                reason: "install-origin-match".to_owned(),
            },
            "install-origin-match".to_owned(),
        )
    };

    PlannedUpdate {
        stable_id: app.stable_id.clone(),
        display_name: app.display_name.clone(),
        selected_channel,
        selection_reason,
    }
}

fn execute_update(
    app: &AppRecord,
    install_home: &Path,
) -> Result<crate::app::add::InstalledApp, String> {
    let query = update_query(app).ok_or_else(|| "missing install source".to_owned())?;
    let requested_scope = app
        .install
        .as_ref()
        .map(|install| install.scope)
        .unwrap_or(InstallScope::User);
    let plan = build_add_plan(&query)
        .map_err(|error| format!("failed to build update plan: {error:?}"))?;

    install_app(&query, &plan, install_home, requested_scope)
        .map_err(|error| format!("failed to install update: {error:?}"))
}

fn update_query(app: &AppRecord) -> Option<String> {
    app.source_input.clone().or_else(|| {
        app.source.as_ref().map(|source| {
            source
                .canonical_locator
                .clone()
                .unwrap_or_else(|| source.locator.clone())
        })
    })
}
