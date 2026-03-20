pub mod cli;
pub mod ui;

use std::env;
use std::path::{Path, PathBuf};

use aim_core::app::add::{
    AddPlan, InstalledApp, build_add_plan, install_app_with_reporter, resolve_requested_scope,
};
use aim_core::app::list::{ListRow, build_list_rows};
use aim_core::app::progress::{NoopReporter, OperationEvent, OperationStage, ProgressReporter};
use aim_core::app::remove::{RemovalResult, remove_registered_app_with_reporter};
use aim_core::app::update::{build_update_plan, execute_updates_with_reporter};
use aim_core::domain::app::AppRecord;
use aim_core::domain::update::{UpdateExecutionResult, UpdatePlan};
use aim_core::registry::model::Registry;
use aim_core::registry::store::RegistryStore;

pub use cli::args::Cli;

pub fn parse() -> Cli {
    <Cli as clap::Parser>::parse()
}

pub fn dispatch(cli: Cli) -> Result<DispatchResult, DispatchError> {
    let mut reporter = NoopReporter;
    dispatch_with_reporter(cli, &mut reporter)
}

pub fn dispatch_with_reporter(
    cli: Cli,
    reporter: &mut impl ProgressReporter,
) -> Result<DispatchResult, DispatchError> {
    let registry_path = registry_path();
    let install_home = install_home(&registry_path);
    let store = RegistryStore::new(registry_path);
    let registry = store.load()?;
    let apps = registry.apps.clone();

    if cli.is_review_update_flow() {
        return Ok(DispatchResult::UpdatePlan(build_update_plan(&apps)?));
    }

    if let Some(command) = cli.command {
        return match command {
            cli::args::Command::List => Ok(DispatchResult::List(build_list_rows(&apps))),
            cli::args::Command::Remove { query } => {
                let removal =
                    remove_registered_app_with_reporter(&query, &apps, &install_home, reporter)?;
                let remaining_apps = removal.remaining_apps.clone();
                reporter.report(&OperationEvent::StageChanged {
                    stage: OperationStage::SaveRegistry,
                    message: "saving registry".to_owned(),
                });
                store.save(&Registry {
                    version: registry.version,
                    apps: remaining_apps,
                })?;
                reporter.report(&OperationEvent::Finished {
                    summary: format!("removed {}", removal.removed.stable_id),
                });
                Ok(DispatchResult::Removed(Box::new(removal)))
            }
            cli::args::Command::Update => {
                let updates = execute_updates_with_reporter(&apps, &install_home, reporter)?;
                let updated_apps = updates.apps.clone();
                reporter.report(&OperationEvent::StageChanged {
                    stage: OperationStage::SaveRegistry,
                    message: "saving registry".to_owned(),
                });
                store.save(&Registry {
                    version: registry.version,
                    apps: updated_apps,
                })?;
                reporter.report(&OperationEvent::Finished {
                    summary: format!(
                        "updated {}, failed {}",
                        updates.updated_count(),
                        updates.failed_count()
                    ),
                });
                Ok(DispatchResult::Updated(Box::new(updates)))
            }
        };
    }

    if let Some(query) = cli.query {
        let requested_scope = resolve_requested_scope(cli.system, cli.user, is_effective_root());
        let mut plan = build_add_plan(&query)?;
        if !plan.interactions.is_empty() {
            match ui::prompt::resolve_add_plan_interactions(plan.clone())? {
                Some(resolved) => {
                    plan = resolved;
                }
                None => return Ok(DispatchResult::PendingAdd(Box::new(plan))),
            }
        }

        let installed =
            install_app_with_reporter(&query, &plan, &install_home, requested_scope, reporter)?;
        let mut updated_apps = registry.apps.clone();
        upsert_app_record(&mut updated_apps, installed.record.clone());
        reporter.report(&OperationEvent::StageChanged {
            stage: OperationStage::SaveRegistry,
            message: "saving registry".to_owned(),
        });
        store.save(&Registry {
            version: registry.version,
            apps: updated_apps,
        })?;
        reporter.report(&OperationEvent::Finished {
            summary: format!("installed {}", installed.record.stable_id),
        });

        return Ok(DispatchResult::Added(Box::new(installed)));
    }

    Ok(DispatchResult::Noop)
}

pub fn render(result: &DispatchResult) -> String {
    ui::render::render_dispatch_result(result)
}

fn registry_path() -> PathBuf {
    if let Some(path) = env::var_os("AIM_REGISTRY_PATH") {
        return PathBuf::from(path);
    }

    let home = env::var_os("HOME").unwrap_or_else(|| ".".into());
    PathBuf::from(home).join(".local/share/aim/registry.toml")
}

#[derive(Debug, Eq, PartialEq)]
pub enum DispatchResult {
    Added(Box<InstalledApp>),
    List(Vec<ListRow>),
    PendingAdd(Box<AddPlan>),
    Removed(Box<RemovalResult>),
    UpdatePlan(UpdatePlan),
    Updated(Box<UpdateExecutionResult>),
    Noop,
}

#[derive(Debug)]
pub enum DispatchError {
    AddPlan(aim_core::app::add::BuildAddPlanError),
    AddInstall(aim_core::app::add::InstallAppError),
    Prompt(ui::prompt::PromptError),
    RemovePlan(aim_core::app::remove::RemoveRegisteredAppError),
    Registry(aim_core::registry::store::RegistryStoreError),
    UpdatePlan(aim_core::app::update::BuildUpdatePlanError),
    UpdateExecution(aim_core::app::update::ExecuteUpdatesError),
}

impl From<aim_core::app::add::BuildAddPlanError> for DispatchError {
    fn from(value: aim_core::app::add::BuildAddPlanError) -> Self {
        Self::AddPlan(value)
    }
}

impl From<aim_core::app::add::InstallAppError> for DispatchError {
    fn from(value: aim_core::app::add::InstallAppError) -> Self {
        Self::AddInstall(value)
    }
}

impl From<ui::prompt::PromptError> for DispatchError {
    fn from(value: ui::prompt::PromptError) -> Self {
        Self::Prompt(value)
    }
}

impl From<aim_core::app::update::BuildUpdatePlanError> for DispatchError {
    fn from(value: aim_core::app::update::BuildUpdatePlanError) -> Self {
        Self::UpdatePlan(value)
    }
}

impl From<aim_core::app::update::ExecuteUpdatesError> for DispatchError {
    fn from(value: aim_core::app::update::ExecuteUpdatesError) -> Self {
        Self::UpdateExecution(value)
    }
}

impl From<aim_core::app::remove::RemoveRegisteredAppError> for DispatchError {
    fn from(value: aim_core::app::remove::RemoveRegisteredAppError) -> Self {
        Self::RemovePlan(value)
    }
}

impl From<aim_core::registry::store::RegistryStoreError> for DispatchError {
    fn from(value: aim_core::registry::store::RegistryStoreError) -> Self {
        Self::Registry(value)
    }
}

fn upsert_app_record(apps: &mut Vec<AppRecord>, record: AppRecord) {
    if let Some(existing) = apps
        .iter_mut()
        .find(|item| item.stable_id == record.stable_id)
    {
        *existing = record;
        return;
    }

    apps.push(record);
}

fn install_home(registry_path: &Path) -> PathBuf {
    if env::var_os("AIM_REGISTRY_PATH").is_some() {
        return registry_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("install-home");
    }

    let home = env::var_os("HOME").unwrap_or_else(|| ".".into());
    PathBuf::from(home)
}

fn is_effective_root() -> bool {
    if let Some(value) = env::var_os("AIM_EFFECTIVE_ROOT") {
        let value = value.to_string_lossy();
        return value == "1" || value.eq_ignore_ascii_case("true");
    }

    #[cfg(unix)]
    unsafe {
        libc::geteuid() == 0
    }

    #[cfg(not(unix))]
    {
        false
    }
}
