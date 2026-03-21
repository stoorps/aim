use crate::adapters::traits::AdapterError;
use crate::app::add::{BuildAddPlanError, build_add_plan, build_add_plan_with};
use crate::app::interaction::InteractionKind;
use crate::domain::app::AppRecord;
use crate::domain::show::{
    AdapterFailureKind, GitHubDiscoveryFailureKind, InstalledShow, MetadataSummary,
    RemoteArtifactSummary, RemoteInteractionSummary, RemoteShow, ShowResult, ShowResultError,
    SourceSummary, TrackedInstallPaths, UpdateChannelSummary, UpdateStrategySummary,
};
use crate::source::github::GitHubTransport;

pub fn build_show_result(
    query: &str,
    installed_apps: &[AppRecord],
) -> Result<ShowResult, ShowResultError> {
    match resolve_installed_show(query, installed_apps) {
        InstalledLookup::Found(app) => Ok(ShowResult::Installed(project_installed_show(app))),
        InstalledLookup::Missing => build_remote_show_result(query),
        InstalledLookup::Ambiguous(matches) => Err(ambiguous_installed_match(query, matches)),
    }
}

pub fn build_installed_show_results(installed_apps: &[AppRecord]) -> Vec<InstalledShow> {
    installed_apps.iter().map(project_installed_show).collect()
}

pub fn build_show_result_with<T: GitHubTransport + ?Sized>(
    query: &str,
    installed_apps: &[AppRecord],
    transport: &T,
) -> Result<ShowResult, ShowResultError> {
    match resolve_installed_show(query, installed_apps) {
        InstalledLookup::Found(app) => Ok(ShowResult::Installed(project_installed_show(app))),
        InstalledLookup::Missing => {
            let plan = build_add_plan_with(query, transport).map_err(ShowResultError::from)?;
            let warnings = collect_metadata_warnings(&plan.metadata);
            let interactions = summarize_interactions(&plan.interactions);
            Ok(ShowResult::Remote(RemoteShow {
                source: project_source_summary(&plan.resolution.source),
                artifact: RemoteArtifactSummary {
                    url: plan.selected_artifact.url,
                    version: optional_version(plan.selected_artifact.version),
                    arch: plan.selected_artifact.arch,
                    trusted_checksum: plan.selected_artifact.trusted_checksum,
                    selection_reason: plan.selected_artifact.selection_reason,
                },
                interactions,
                warnings,
            }))
        }
        InstalledLookup::Ambiguous(matches) => Err(ambiguous_installed_match(query, matches)),
    }
}

fn build_remote_show_result(query: &str) -> Result<ShowResult, ShowResultError> {
    let plan = build_add_plan(query).map_err(ShowResultError::from)?;
    let warnings = collect_metadata_warnings(&plan.metadata);
    let interactions = summarize_interactions(&plan.interactions);

    Ok(ShowResult::Remote(RemoteShow {
        source: project_source_summary(&plan.resolution.source),
        artifact: RemoteArtifactSummary {
            url: plan.selected_artifact.url,
            version: optional_version(plan.selected_artifact.version),
            arch: plan.selected_artifact.arch,
            trusted_checksum: plan.selected_artifact.trusted_checksum,
            selection_reason: plan.selected_artifact.selection_reason,
        },
        interactions,
        warnings,
    }))
}

fn ambiguous_installed_match(query: &str, matches: Vec<String>) -> ShowResultError {
    ShowResultError::AmbiguousInstalledMatch {
        query: query.to_owned(),
        matches,
    }
}

enum InstalledLookup<'a> {
    Found(&'a AppRecord),
    Missing,
    Ambiguous(Vec<String>),
}

fn resolve_installed_show<'a>(query: &str, installed_apps: &'a [AppRecord]) -> InstalledLookup<'a> {
    let normalized_query = normalize_lookup(query);
    let matches = installed_apps
        .iter()
        .filter(|app| app_matches_installed_query(app, &normalized_query))
        .collect::<Vec<_>>();

    match matches.as_slice() {
        [] => InstalledLookup::Missing,
        [app] => InstalledLookup::Found(app),
        _ => InstalledLookup::Ambiguous(
            matches
                .iter()
                .map(|app| format!("{} ({})", app.display_name, app.stable_id))
                .collect(),
        ),
    }
}

fn app_matches_installed_query(app: &AppRecord, normalized_query: &str) -> bool {
    let mut candidates = vec![
        normalize_lookup(&app.stable_id),
        normalize_lookup(&app.display_name),
    ];

    if let Some(source_input) = app.source_input.as_deref() {
        candidates.push(normalize_lookup(source_input));
    }

    if let Some(source) = app.source.as_ref() {
        candidates.push(normalize_lookup(&source.locator));
        if let Some(canonical_locator) = source.canonical_locator.as_deref() {
            candidates.push(normalize_lookup(canonical_locator));
        }
    }

    candidates
        .iter()
        .any(|candidate| candidate == normalized_query)
}

fn normalize_lookup(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn optional_version(version: String) -> Option<String> {
    (version != "unresolved").then_some(version)
}

fn collect_metadata_warnings(metadata: &[crate::domain::update::ParsedMetadata]) -> Vec<String> {
    metadata
        .iter()
        .flat_map(|item| item.warnings.iter().cloned())
        .collect()
}

fn project_installed_show(app: &AppRecord) -> InstalledShow {
    InstalledShow {
        stable_id: app.stable_id.clone(),
        display_name: app.display_name.clone(),
        installed_version: app.installed_version.clone().and_then(optional_version),
        source_input: app.source_input.clone(),
        source: app.source.as_ref().map(project_source_summary),
        install_scope: app.install.as_ref().map(|install| install.scope),
        tracked_paths: TrackedInstallPaths {
            payload_path: app
                .install
                .as_ref()
                .and_then(|install| install.payload_path.clone()),
            desktop_entry_path: app
                .install
                .as_ref()
                .and_then(|install| install.desktop_entry_path.clone()),
            icon_path: app
                .install
                .as_ref()
                .and_then(|install| install.icon_path.clone()),
        },
        update_strategy: app
            .update_strategy
            .as_ref()
            .map(|strategy| UpdateStrategySummary {
                preferred: UpdateChannelSummary {
                    kind: strategy.preferred.kind,
                    locator: strategy.preferred.locator.clone(),
                    reason: strategy.preferred.reason.clone(),
                },
                alternates: strategy
                    .alternates
                    .iter()
                    .map(|alternate| UpdateChannelSummary {
                        kind: alternate.kind,
                        locator: alternate.locator.clone(),
                        reason: alternate.reason.clone(),
                    })
                    .collect(),
            }),
        metadata: app
            .metadata
            .iter()
            .map(|item| MetadataSummary {
                kind: item.kind,
                version: item.hints.version.clone(),
                primary_download: item.hints.primary_download.clone(),
                checksum: item.hints.checksum.clone(),
                architecture: item.hints.architecture.clone(),
                channel_label: item.hints.channel_label.clone(),
                warnings: item.warnings.clone(),
            })
            .collect(),
    }
}

fn project_source_summary(source: &crate::domain::source::SourceRef) -> SourceSummary {
    SourceSummary {
        kind: source.kind,
        locator: source.locator.clone(),
        canonical_locator: source.canonical_locator.clone(),
    }
}

fn summarize_interactions(
    interactions: &[crate::app::interaction::InteractionRequest],
) -> Vec<RemoteInteractionSummary> {
    interactions
        .iter()
        .filter_map(|interaction| match &interaction.kind {
            InteractionKind::SelectRegisteredApp { query, matches } => {
                let _ = query;
                let _ = matches;
                None
            }
            InteractionKind::ChooseTrackingPreference {
                requested_version,
                latest_version,
            } => Some(RemoteInteractionSummary::ChooseTrackingPreference {
                requested_version: requested_version.clone(),
                latest_version: latest_version.clone(),
            }),
            InteractionKind::SelectArtifact { candidates } => {
                Some(RemoteInteractionSummary::SelectArtifact {
                    candidate_count: candidates.len(),
                })
            }
        })
        .collect()
}

impl From<BuildAddPlanError> for ShowResultError {
    fn from(value: BuildAddPlanError) -> Self {
        match value {
            BuildAddPlanError::Query(_) => Self::UnsupportedQuery,
            BuildAddPlanError::InsecureHttpSource { .. } => Self::InsecureHttpSource,
            BuildAddPlanError::NoInstallableArtifact { source } => Self::NoInstallableArtifact {
                source: project_source_summary(&source),
            },
            BuildAddPlanError::Adapter(id, error) => Self::AdapterResolutionFailed {
                adapter_id: id.to_owned(),
                kind: match &error {
                    AdapterError::UnsupportedQuery => AdapterFailureKind::UnsupportedQuery,
                    AdapterError::UnsupportedSource => AdapterFailureKind::UnsupportedSource,
                    AdapterError::ResolutionFailed(_) => AdapterFailureKind::ResolutionFailed,
                },
                detail: match error {
                    AdapterError::ResolutionFailed(reason) => Some(reason),
                    _ => None,
                },
            },
            BuildAddPlanError::GitHubDiscovery(error) => Self::GitHubDiscoveryFailed {
                kind: match &error {
                    crate::source::github::GitHubDiscoveryError::Unsupported => {
                        GitHubDiscoveryFailureKind::Unsupported
                    }
                    crate::source::github::GitHubDiscoveryError::FixtureDocumentMissing(_) => {
                        GitHubDiscoveryFailureKind::FixtureDocumentMissing
                    }
                    crate::source::github::GitHubDiscoveryError::NoReleases { .. } => {
                        GitHubDiscoveryFailureKind::NoReleases
                    }
                    crate::source::github::GitHubDiscoveryError::Transport(_) => {
                        GitHubDiscoveryFailureKind::Transport
                    }
                },
                detail: match error {
                    crate::source::github::GitHubDiscoveryError::FixtureDocumentMissing(url) => {
                        Some(url)
                    }
                    crate::source::github::GitHubDiscoveryError::NoReleases { repo } => Some(repo),
                    _ => None,
                },
            },
            BuildAddPlanError::NoCandidates => Self::NoInstallableCandidates,
        }
    }
}
