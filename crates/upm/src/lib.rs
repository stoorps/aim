pub mod cli;
pub mod config;
pub mod providers;
pub mod ui;

use std::collections::{HashMap, HashSet};
use std::env;
use std::path::{Path, PathBuf};

use upm_core::app::add::{AddPlan, AddSecurityPolicy, InstalledApp, resolve_requested_scope};
use upm_core::app::list::ListRow;
use upm_core::app::progress::{
    NoopReporter, OperationEvent, OperationKind, OperationStage, ProgressReporter,
};
use upm_core::app::remove::{RemovalResult, remove_registered_app_with_reporter};
use upm_core::domain::app::AppRecord;
use upm_core::domain::search::{SearchQuery, SearchResults};
use upm_core::domain::show::{InstalledShow, ShowResult};
use upm_core::domain::update::{UpdateExecutionResult, UpdatePlan};
use upm_core::registry::store::RegistryStore;

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
    dispatch_with_reporter_and_config(cli, &crate::config::CliConfig::default(), reporter)
}

pub fn dispatch_with_reporter_and_config(
    cli: Cli,
    config: &crate::config::CliConfig,
    reporter: &mut impl ProgressReporter,
) -> Result<DispatchResult, DispatchError> {
    let registry_path = registry_path();
    let install_home = install_home(&registry_path);
    let store = RegistryStore::new(registry_path);
    let registry = store.load()?;
    let apps = registry.apps.clone();
    let app = providers::application();

    if cli.is_review_update_flow() {
        return Ok(DispatchResult::UpdatePlan(app.build_update_plan(&apps)?));
    }

    if let Some(command) = cli.command {
        return match command {
            cli::args::Command::List => Ok(DispatchResult::List(app.list(&apps))),
            cli::args::Command::Remove { query } => {
                let removal =
                    remove_registered_app_with_reporter(&query, &apps, &install_home, reporter)?;
                reporter.report(&OperationEvent::StageChanged {
                    stage: OperationStage::SaveRegistry,
                    message: "saving registry".to_owned(),
                });
                store.mutate_exclusive(|latest| {
                    remove_app_record(&mut latest.apps, &removal.removed.stable_id);
                })?;
                reporter.report(&OperationEvent::Finished {
                    summary: format!("removed {}", removal.removed.stable_id),
                });
                Ok(DispatchResult::Removed(Box::new(removal)))
            }
            cli::args::Command::Search { query } => {
                reporter.report(&OperationEvent::Started {
                    kind: OperationKind::Search,
                    label: query.clone(),
                });
                let results = app.search(&SearchQuery::new(&query), &apps)?;
                reporter.report(&OperationEvent::Finished {
                    summary: format!("search complete: {} remote hits", results.remote_hits.len()),
                });
                Ok(DispatchResult::Search(results))
            }
            cli::args::Command::Show { value } => match value {
                Some(value) => {
                    let result = app.show(&value, &apps)?;
                    Ok(DispatchResult::Show(Box::new(result)))
                }
                None => Ok(DispatchResult::ShowAll(app.show_all(&apps))),
            },
            cli::args::Command::Update => {
                let updates = app.execute_updates(
                    &apps,
                    &install_home,
                    reporter,
                    AddSecurityPolicy {
                        allow_http_user_sources: config.allow_http,
                    },
                )?;
                reporter.report(&OperationEvent::StageChanged {
                    stage: OperationStage::SaveRegistry,
                    message: "saving registry".to_owned(),
                });
                store.mutate_exclusive(|latest| {
                    merge_updated_app_records(&mut latest.apps, &apps, &updates.apps);
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
        let plan_result = app.build_add_plan_with_reporter(
            &query,
            reporter,
            AddSecurityPolicy {
                allow_http_user_sources: config.allow_http,
            },
        );
        let mut plan = match plan_result {
            Ok(plan) => plan,
            Err(
                upm_core::app::add::BuildAddPlanError::Query(
                    upm_core::app::query::ResolveQueryError::Unsupported,
                )
                | upm_core::app::add::BuildAddPlanError::NoInstallableArtifact { .. },
            ) => {
                reporter.report(&OperationEvent::Started {
                    kind: OperationKind::Search,
                    label: query.clone(),
                });
                let results = app.search(&SearchQuery::new(&query), &apps)?;
                reporter.report(&OperationEvent::Finished {
                    summary: format!("search complete: {} remote hits", results.remote_hits.len()),
                });
                return Ok(DispatchResult::Search(results));
            }
            Err(error) => return Err(error.into()),
        };
        if !plan.interactions.is_empty() {
            match ui::prompt::resolve_add_plan_interactions(plan.clone())? {
                Some(resolved) => {
                    plan = resolved;
                }
                None => return Ok(DispatchResult::PendingAdd(Box::new(plan))),
            }
        }

        let installed = app.install_app(&query, &plan, &install_home, requested_scope, reporter)?;
        reporter.report(&OperationEvent::StageChanged {
            stage: OperationStage::SaveRegistry,
            message: "saving registry".to_owned(),
        });
        store.mutate_exclusive(|latest| {
            upsert_app_record(&mut latest.apps, installed.record.clone());
        })?;
        reporter.report(&OperationEvent::Finished {
            summary: format!("installed {}", installed.record.stable_id),
        });

        return Ok(DispatchResult::Added(Box::new(installed)));
    }

    Ok(DispatchResult::Noop)
}

pub fn render(result: &DispatchResult) -> String {
    render_with_config(result, &config::CliConfig::default())
}

pub fn render_with_config(result: &DispatchResult, config: &config::CliConfig) -> String {
    ui::render::render_dispatch_result_with_config(result, config)
}

pub fn default_registry_path() -> PathBuf {
    if let Some(path) = env::var_os("UPM_REGISTRY_PATH") {
        return PathBuf::from(path);
    }

    let home = env::var_os("HOME").unwrap_or_else(|| ".".into());
    PathBuf::from(home).join(".local/share/upm/registry.toml")
}

fn registry_path() -> PathBuf {
    default_registry_path()
}

#[derive(Debug, Eq, PartialEq)]
pub enum DispatchResult {
    Added(Box<InstalledApp>),
    List(Vec<ListRow>),
    PendingAdd(Box<AddPlan>),
    Removed(Box<RemovalResult>),
    Search(SearchResults),
    Show(Box<ShowResult>),
    ShowAll(Vec<InstalledShow>),
    UpdatePlan(UpdatePlan),
    Updated(Box<UpdateExecutionResult>),
    Noop,
}

#[derive(Debug)]
pub enum DispatchError {
    AddPlan(upm_core::app::add::BuildAddPlanError),
    AddInstall(upm_core::app::add::InstallAppError),
    Prompt(ui::prompt::PromptError),
    RemovePlan(upm_core::app::remove::RemoveRegisteredAppError),
    Registry(upm_core::registry::store::RegistryStoreError),
    Search(upm_core::app::search::SearchError),
    Show(upm_core::domain::show::ShowResultError),
    UpdatePlan(upm_core::app::update::BuildUpdatePlanError),
    UpdateExecution(upm_core::app::update::ExecuteUpdatesError),
}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddPlan(error) => match error {
                upm_core::app::add::BuildAddPlanError::Query(
                    upm_core::app::query::ResolveQueryError::Unsupported,
                ) => write!(f, "unsupported source query"),
                upm_core::app::add::BuildAddPlanError::InsecureHttpSource { .. } => write!(
                    f,
                    "insecure HTTP sources are disabled; set allow_http = true to permit them"
                ),
                upm_core::app::add::BuildAddPlanError::NoInstallableArtifact { source } => write!(
                    f,
                    "no installable artifact found for {} {}",
                    source.kind.as_str(),
                    source.locator
                ),
                upm_core::app::add::BuildAddPlanError::Adapter(id, error) => match error {
                    upm_core::adapters::traits::AdapterError::UnsupportedQuery => {
                        write!(f, "{id} does not support this query")
                    }
                    upm_core::adapters::traits::AdapterError::UnsupportedSource => {
                        write!(f, "{id} does not support this source")
                    }
                    upm_core::adapters::traits::AdapterError::ResolutionFailed(reason) => {
                        write!(f, "{id} resolution failed: {reason}")
                    }
                },
                upm_core::app::add::BuildAddPlanError::GitHubDiscovery(error) => {
                    write!(f, "github discovery failed: {error:?}")
                }
                upm_core::app::add::BuildAddPlanError::NoCandidates => {
                    write!(f, "no installable candidates found")
                }
            },
            Self::AddInstall(error) => write!(f, "install failed: {}", render_install_error(error)),
            Self::Prompt(error) => write!(f, "prompt failed: {error:?}"),
            Self::RemovePlan(error) => write!(f, "remove failed: {error:?}"),
            Self::Registry(error) => write!(f, "registry failed: {error:?}"),
            Self::Search(error) => write!(f, "search failed: {error:?}"),
            Self::Show(error) => match error {
                upm_core::domain::show::ShowResultError::AmbiguousInstalledMatch {
                    query,
                    matches,
                } => write!(
                    f,
                    "multiple installed apps match {query}: {}",
                    matches.join(", ")
                ),
                upm_core::domain::show::ShowResultError::UnsupportedQuery => {
                    write!(f, "unsupported source query")
                }
                upm_core::domain::show::ShowResultError::InsecureHttpSource => write!(
                    f,
                    "insecure HTTP sources are disabled; set allow_http = true to permit them"
                ),
                upm_core::domain::show::ShowResultError::NoInstallableArtifact { source } => {
                    write!(
                        f,
                        "no installable artifact found for {} {}",
                        source.kind.as_str(),
                        source.locator
                    )
                }
                upm_core::domain::show::ShowResultError::AdapterResolutionFailed {
                    adapter_id,
                    kind,
                    detail,
                } => match kind {
                    upm_core::domain::show::AdapterFailureKind::UnsupportedQuery => {
                        write!(f, "{adapter_id} does not support this query")
                    }
                    upm_core::domain::show::AdapterFailureKind::UnsupportedSource => {
                        write!(f, "{adapter_id} does not support this source")
                    }
                    upm_core::domain::show::AdapterFailureKind::ResolutionFailed => {
                        if let Some(detail) = detail {
                            write!(f, "{adapter_id} resolution failed: {detail}")
                        } else {
                            write!(f, "{adapter_id} resolution failed")
                        }
                    }
                },
                upm_core::domain::show::ShowResultError::GitHubDiscoveryFailed {
                    kind,
                    detail,
                } => match (kind, detail) {
                    (
                        upm_core::domain::show::GitHubDiscoveryFailureKind::FixtureDocumentMissing,
                        Some(detail),
                    ) => write!(f, "github discovery failed: missing fixture document {detail}"),
                    (
                        upm_core::domain::show::GitHubDiscoveryFailureKind::NoReleases,
                        Some(detail),
                    ) => write!(f, "github discovery failed: no releases for {detail}"),
                    (upm_core::domain::show::GitHubDiscoveryFailureKind::Unsupported, _) => {
                        write!(f, "github discovery failed: unsupported source")
                    }
                    (upm_core::domain::show::GitHubDiscoveryFailureKind::Transport, _) => {
                        write!(f, "github discovery failed: transport error")
                    }
                    _ => write!(f, "github discovery failed"),
                },
                upm_core::domain::show::ShowResultError::NoInstallableCandidates => {
                    write!(f, "no installable candidates found")
                }
            },
            Self::UpdatePlan(error) => write!(f, "update planning failed: {error:?}"),
            Self::UpdateExecution(error) => write!(f, "update execution failed: {error:?}"),
        }
    }
}

fn render_install_error(error: &upm_core::app::add::InstallAppError) -> String {
    match error {
        upm_core::app::add::InstallAppError::Materialize(error) => format!("{error:?}"),
        upm_core::app::add::InstallAppError::Policy(error) => error.clone(),
        upm_core::app::add::InstallAppError::Download(error) => error.to_string(),
        upm_core::app::add::InstallAppError::DownloadIo(error) => error.to_string(),
        upm_core::app::add::InstallAppError::HostProbe(error) => error.to_string(),
        upm_core::app::add::InstallAppError::Install(error) => error.to_string(),
    }
}

impl From<upm_core::app::add::BuildAddPlanError> for DispatchError {
    fn from(value: upm_core::app::add::BuildAddPlanError) -> Self {
        Self::AddPlan(value)
    }
}

impl From<upm_core::app::add::InstallAppError> for DispatchError {
    fn from(value: upm_core::app::add::InstallAppError) -> Self {
        Self::AddInstall(value)
    }
}

impl From<ui::prompt::PromptError> for DispatchError {
    fn from(value: ui::prompt::PromptError) -> Self {
        Self::Prompt(value)
    }
}

impl From<upm_core::app::update::BuildUpdatePlanError> for DispatchError {
    fn from(value: upm_core::app::update::BuildUpdatePlanError) -> Self {
        Self::UpdatePlan(value)
    }
}

impl From<upm_core::app::update::ExecuteUpdatesError> for DispatchError {
    fn from(value: upm_core::app::update::ExecuteUpdatesError) -> Self {
        Self::UpdateExecution(value)
    }
}

impl From<upm_core::app::remove::RemoveRegisteredAppError> for DispatchError {
    fn from(value: upm_core::app::remove::RemoveRegisteredAppError) -> Self {
        Self::RemovePlan(value)
    }
}

impl From<upm_core::registry::store::RegistryStoreError> for DispatchError {
    fn from(value: upm_core::registry::store::RegistryStoreError) -> Self {
        Self::Registry(value)
    }
}

impl From<upm_core::app::search::SearchError> for DispatchError {
    fn from(value: upm_core::app::search::SearchError) -> Self {
        Self::Search(value)
    }
}

impl From<upm_core::domain::show::ShowResultError> for DispatchError {
    fn from(value: upm_core::domain::show::ShowResultError) -> Self {
        Self::Show(value)
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

fn remove_app_record(apps: &mut Vec<AppRecord>, stable_id: &str) {
    apps.retain(|app| app.stable_id != stable_id);
}

fn merge_updated_app_records(
    latest_apps: &mut [AppRecord],
    original_apps: &[AppRecord],
    updated_apps: &[AppRecord],
) {
    let original_ids = original_apps
        .iter()
        .map(|app| app.stable_id.as_str())
        .collect::<HashSet<_>>();
    let updated_by_id = updated_apps
        .iter()
        .map(|app| (app.stable_id.as_str(), app.clone()))
        .collect::<HashMap<_, _>>();

    for app in latest_apps.iter_mut() {
        if original_ids.contains(app.stable_id.as_str())
            && let Some(updated) = updated_by_id.get(app.stable_id.as_str())
        {
            *app = updated.clone();
        }
    }
}

fn install_home(registry_path: &Path) -> PathBuf {
    if env::var_os("UPM_REGISTRY_PATH").is_some() {
        return registry_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("install-home");
    }

    let home = env::var_os("HOME").unwrap_or_else(|| ".".into());
    PathBuf::from(home)
}

fn is_effective_root() -> bool {
    if let Some(value) = env::var_os("UPM_EFFECTIVE_ROOT") {
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
