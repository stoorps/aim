use std::fs;
use std::path::{Path, PathBuf};

use crate::app::add::{build_add_plan, install_app_with_reporter};
use crate::app::progress::{
    NoopReporter, OperationEvent, OperationKind, OperationStage, ProgressReporter,
};
use crate::domain::app::{AppRecord, InstallScope};
use crate::domain::source::SourceKind;
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
    let mut reporter = NoopReporter;
    execute_updates_with_reporter(apps, install_home, &mut reporter)
}

pub fn execute_updates_with_reporter(
    apps: &[AppRecord],
    install_home: &Path,
    reporter: &mut impl ProgressReporter,
) -> Result<UpdateExecutionResult, ExecuteUpdatesError> {
    reporter.report(&OperationEvent::Started {
        kind: OperationKind::UpdateBatch,
        label: format!("{} apps", apps.len()),
    });
    let mut updated_apps = Vec::with_capacity(apps.len());
    let mut items = Vec::with_capacity(apps.len());

    for app in apps {
        reporter.report(&OperationEvent::Started {
            kind: OperationKind::UpdateItem,
            label: app.stable_id.clone(),
        });
        match execute_update(app, install_home, reporter) {
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
                reporter.report(&OperationEvent::Finished {
                    summary: format!("updated {}", app.stable_id),
                });
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

    let result = UpdateExecutionResult {
        apps: updated_apps,
        items,
    };

    reporter.report(&OperationEvent::Finished {
        summary: format!(
            "updated {}, failed {}",
            result.updated_count(),
            result.failed_count()
        ),
    });

    Ok(result)
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
            fallback_channel_preference(app),
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

fn fallback_channel_preference(app: &AppRecord) -> ChannelPreference {
    let Some(source) = app.source.as_ref() else {
        return ChannelPreference {
            kind: UpdateChannelKind::GitHubReleases,
            locator: app.stable_id.clone(),
            reason: "install-origin-match".to_owned(),
        };
    };

    let (kind, locator) = match source.kind {
        SourceKind::GitHub => (
            UpdateChannelKind::GitHubReleases,
            source
                .canonical_locator
                .clone()
                .unwrap_or_else(|| source.locator.clone()),
        ),
        SourceKind::GitLab | SourceKind::SourceForge | SourceKind::DirectUrl | SourceKind::File => {
            (UpdateChannelKind::DirectAsset, source.locator.clone())
        }
    };

    ChannelPreference {
        kind,
        locator,
        reason: "install-origin-match".to_owned(),
    }
}

fn execute_update(
    app: &AppRecord,
    install_home: &Path,
    reporter: &mut impl ProgressReporter,
) -> Result<crate::app::add::InstalledApp, String> {
    reporter.report(&OperationEvent::StageChanged {
        stage: OperationStage::ResolveQuery,
        message: format!("resolving {}", app.stable_id),
    });
    let query = update_query(app).ok_or_else(|| {
        let reason = "missing install source".to_owned();
        reporter.report(&OperationEvent::Failed {
            stage: OperationStage::ResolveQuery,
            reason: reason.clone(),
        });
        reason
    })?;
    let requested_scope = app
        .install
        .as_ref()
        .map(|install| install.scope)
        .unwrap_or(InstallScope::User);
    let plan = build_add_plan(&query).map_err(|error| {
        let reason = format!("failed to build update plan: {error:?}");
        reporter.report(&OperationEvent::Failed {
            stage: OperationStage::ResolveQuery,
            reason: reason.clone(),
        });
        reason
    })?;

    let rollback = stage_existing_installation(app, install_home).inspect_err(|reason| {
        reporter.report(&OperationEvent::Failed {
            stage: OperationStage::StagePayload,
            reason: reason.clone(),
        });
    })?;

    install_app_with_reporter(&query, &plan, install_home, requested_scope, reporter)
        .map_err(|error| {
            let install_reason = format!("failed to install update: {error:?}");
            let reason = match rollback.as_ref() {
                Some(rollback) => match rollback.restore() {
                    Ok(()) => format!("{install_reason}; restored previous installation"),
                    Err(restore_reason) => {
                        format!("{install_reason}; rollback restore failed: {restore_reason}")
                    }
                },
                None => install_reason,
            };
            reporter.report(&OperationEvent::Failed {
                stage: OperationStage::Finalize,
                reason: reason.clone(),
            });
            reason
        })
        .inspect(|_| {
            if let Some(rollback) = rollback.as_ref() {
                let _ = rollback.cleanup();
            }
        })
}

fn update_query(app: &AppRecord) -> Option<String> {
    if let Some(source) = app.source.as_ref()
        && source.kind == SourceKind::SourceForge
    {
        return Some(source.locator.clone());
    }

    app.source_input.clone().or_else(|| {
        app.source.as_ref().map(|source| {
            source
                .canonical_locator
                .clone()
                .unwrap_or_else(|| source.locator.clone())
        })
    })
}

fn stage_existing_installation(
    app: &AppRecord,
    install_home: &Path,
) -> Result<Option<RollbackState>, String> {
    let Some(install) = app.install.as_ref() else {
        return Ok(None);
    };

    let tracked_paths = [
        install.payload_path.as_deref(),
        install.desktop_entry_path.as_deref(),
        install.icon_path.as_deref(),
    ]
    .into_iter()
    .flatten()
    .map(PathBuf::from)
    .filter(|path| path.exists())
    .collect::<Vec<_>>();

    if tracked_paths.is_empty() {
        return Ok(None);
    }

    let stage_dir = install_home
        .join(".local/share/aim/rollback")
        .join(&app.stable_id);
    fs::create_dir_all(&stage_dir)
        .map_err(|error| format!("failed to create rollback staging directory: {error}"))?;

    let mut entries = Vec::with_capacity(tracked_paths.len());
    for original_path in tracked_paths {
        let backup_path = stage_dir.join(
            original_path
                .file_name()
                .map(|name| name.to_os_string())
                .unwrap_or_default(),
        );
        fs::rename(&original_path, &backup_path).map_err(|error| {
            format!(
                "failed to stage existing install file {}: {error}",
                original_path.display()
            )
        })?;
        entries.push(RollbackEntry {
            original_path,
            backup_path,
        });
    }

    Ok(Some(RollbackState { stage_dir, entries }))
}

struct RollbackState {
    stage_dir: PathBuf,
    entries: Vec<RollbackEntry>,
}

impl RollbackState {
    fn restore(&self) -> Result<(), String> {
        for entry in &self.entries {
            if let Some(parent) = entry.original_path.parent() {
                fs::create_dir_all(parent).map_err(|error| {
                    format!(
                        "failed to recreate rollback parent {}: {error}",
                        parent.display()
                    )
                })?;
            }
            fs::rename(&entry.backup_path, &entry.original_path).map_err(|error| {
                format!(
                    "failed to restore {}: {error}",
                    entry.original_path.display()
                )
            })?;
        }
        self.cleanup()
    }

    fn cleanup(&self) -> Result<(), String> {
        if self.stage_dir.exists() {
            fs::remove_dir_all(&self.stage_dir).map_err(|error| {
                format!(
                    "failed to remove rollback staging directory {}: {error}",
                    self.stage_dir.display()
                )
            })?;
        }
        if let Some(parent) = self.stage_dir.parent()
            && parent.exists()
            && fs::read_dir(parent)
                .map_err(|error| {
                    format!(
                        "failed to inspect rollback parent directory {}: {error}",
                        parent.display()
                    )
                })?
                .next()
                .is_none()
        {
            fs::remove_dir(parent).map_err(|error| {
                format!(
                    "failed to remove rollback parent directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        Ok(())
    }
}

struct RollbackEntry {
    original_path: PathBuf,
    backup_path: PathBuf,
}
