pub mod cli;
pub mod ui;

use std::env;
use std::path::PathBuf;

use aim_core::app::add::{AddPlan, build_add_plan, materialize_app_record};
use aim_core::app::list::{ListRow, build_list_rows};
use aim_core::app::remove::remove_registered_app;
use aim_core::app::update::build_update_plan;
use aim_core::domain::app::AppRecord;
use aim_core::domain::source::SourceRef;
use aim_core::domain::update::{ArtifactCandidate, UpdatePlan};
use aim_core::registry::model::Registry;
use aim_core::registry::store::RegistryStore;

pub use cli::args::Cli;

pub fn parse() -> Cli {
    <Cli as clap::Parser>::parse()
}

pub fn dispatch(cli: Cli) -> Result<DispatchResult, DispatchError> {
    let registry_path = registry_path();
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
                let removal = remove_registered_app(&query, &apps)?;
                store.save(&Registry {
                    version: registry.version,
                    apps: removal.remaining_apps,
                })?;
                Ok(DispatchResult::Removed(removal.removed.display_name))
            }
            cli::args::Command::Update => Ok(DispatchResult::UpdatePlan(build_update_plan(&apps)?)),
        };
    }

    if let Some(query) = cli.query {
        let mut plan = build_add_plan(&query)?;
        if !plan.interactions.is_empty() {
            match ui::prompt::resolve_add_plan_interactions(plan.clone())? {
                Some(resolved) => {
                    plan = resolved;
                }
                None => return Ok(DispatchResult::PendingAdd(plan)),
            }
        }

        let record = materialize_app_record(&query, &plan)?;
        let mut updated_apps = registry.apps.clone();
        upsert_app_record(&mut updated_apps, record.clone());
        store.save(&Registry {
            version: registry.version,
            apps: updated_apps,
        })?;

        return Ok(DispatchResult::Added(AddedApp {
            record,
            selected_artifact: plan.selected_artifact,
            source: plan.resolution.source,
        }));
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
    Added(AddedApp),
    List(Vec<ListRow>),
    PendingAdd(AddPlan),
    Removed(String),
    UpdatePlan(UpdatePlan),
    Noop,
}

#[derive(Debug, Eq, PartialEq)]
pub struct AddedApp {
    pub record: AppRecord,
    pub selected_artifact: ArtifactCandidate,
    pub source: SourceRef,
}

#[derive(Debug)]
pub enum DispatchError {
    AddPlan(aim_core::app::add::BuildAddPlanError),
    AddRecord(aim_core::app::add::MaterializeAddRecordError),
    Prompt(ui::prompt::PromptError),
    RemovePlan(aim_core::app::remove::ResolveRegisteredAppError),
    Registry(aim_core::registry::store::RegistryStoreError),
    UpdatePlan(aim_core::app::update::BuildUpdatePlanError),
}

impl From<aim_core::app::add::BuildAddPlanError> for DispatchError {
    fn from(value: aim_core::app::add::BuildAddPlanError) -> Self {
        Self::AddPlan(value)
    }
}

impl From<aim_core::app::add::MaterializeAddRecordError> for DispatchError {
    fn from(value: aim_core::app::add::MaterializeAddRecordError) -> Self {
        Self::AddRecord(value)
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

impl From<aim_core::app::remove::ResolveRegisteredAppError> for DispatchError {
    fn from(value: aim_core::app::remove::ResolveRegisteredAppError) -> Self {
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
