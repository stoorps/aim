use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::adapters::traits::AdapterResolution;
use crate::app::identity::{IdentityFallback, ResolveIdentityError, resolve_identity};
use crate::app::interaction::{InteractionKind, InteractionRequest};
use crate::app::progress::{
    NoopReporter, OperationEvent, OperationKind, OperationStage, ProgressReporter,
};
use crate::app::query::{ResolveQueryError, resolve_query};
use crate::app::scope::{ScopeOverride, resolve_install_scope_with_default};
use crate::domain::app::{AppRecord, InstallMetadata, InstallScope};
use crate::domain::source::{NormalizedSourceKind, ResolvedRelease, SourceKind};
use crate::domain::update::{ArtifactCandidate, ParsedMetadata, UpdateChannelKind, UpdateStrategy};
use crate::integration::install::{
    InstallOutcome, InstallRequest, execute_install, staged_appimage_path,
};
use crate::integration::policy::{IntegrationMode, resolve_install_policy};
use crate::metadata::parse_document;
use crate::platform::probe_live_host;
use crate::source::github::{
    GitHubDiscoveryError, GitHubTransport, discover_github_candidates_with, http_client_policy,
};
use crate::update::channels::build_channels;
use crate::update::ranking::{rank_channels, select_artifact, to_preference};

const FIXTURE_MODE_ENV: &str = "AIM_GITHUB_FIXTURE_MODE";

pub fn build_add_plan(query: &str) -> Result<AddPlan, BuildAddPlanError> {
    let transport = crate::source::github::default_transport();
    build_add_plan_with(query, transport.as_ref())
}

pub fn build_add_plan_with<T: GitHubTransport + ?Sized>(
    query: &str,
    transport: &T,
) -> Result<AddPlan, BuildAddPlanError> {
    let source = resolve_query(query).map_err(BuildAddPlanError::Query)?;

    let mut interactions = Vec::new();
    let mut parsed_metadata = Vec::new();
    let (resolution, selected_artifact, update_strategy) = match source.kind {
        SourceKind::GitHub => {
            let discovery = discover_github_candidates_with(&source, transport)
                .map_err(BuildAddPlanError::GitHubDiscovery)?;
            for document in &discovery.metadata_documents {
                parsed_metadata
                    .push(parse_document(document).expect("metadata parsing is infallible"));
            }

            let ranked = rank_channels(&build_channels(&discovery, &parsed_metadata));
            let preferred = ranked
                .first()
                .cloned()
                .ok_or(BuildAddPlanError::NoCandidates)?;
            let strategy = UpdateStrategy {
                preferred: to_preference(&preferred),
                alternates: ranked.iter().skip(1).map(to_preference).collect(),
            };
            let metadata_hints = parsed_metadata
                .iter()
                .find(|item| item.hints.primary_download.is_some())
                .map(|item| &item.hints);
            let artifact = select_artifact(&preferred, metadata_hints);

            if discovery.requested_is_older_release {
                interactions.push(InteractionRequest {
                    key: "tracking-preference".to_owned(),
                    kind: InteractionKind::ChooseTrackingPreference {
                        requested_version: source.requested_tag.clone().unwrap_or_default(),
                        latest_version: discovery
                            .releases
                            .first()
                            .map(|release| release.tag.clone())
                            .unwrap_or_default(),
                    },
                });
            }

            (
                AdapterResolution {
                    source: source.clone(),
                    release: ResolvedRelease {
                        version: artifact.version.clone(),
                        prerelease: false,
                    },
                },
                artifact,
                strategy,
            )
        }
        _ => {
            let resolution = AdapterResolution {
                source: source.clone(),
                release: ResolvedRelease {
                    version: "unresolved".to_owned(),
                    prerelease: false,
                },
            };
            let artifact = ArtifactCandidate {
                url: source.locator.clone(),
                version: "unresolved".to_owned(),
                arch: None,
                trusted_checksum: None,
                selection_reason: "heuristic-match".to_owned(),
            };
            let strategy = UpdateStrategy {
                preferred: crate::domain::update::ChannelPreference {
                    kind: crate::domain::update::UpdateChannelKind::DirectAsset,
                    locator: source.locator.clone(),
                    reason: "heuristic-match".to_owned(),
                },
                alternates: Vec::new(),
            };
            (resolution, artifact, strategy)
        }
    };

    Ok(AddPlan {
        resolution,
        selected_artifact,
        interactions,
        update_strategy,
        metadata: parsed_metadata,
    })
}

pub fn prefer_latest_tracking(mut plan: AddPlan) -> AddPlan {
    if let Some(index) = plan
        .update_strategy
        .alternates
        .iter()
        .position(|item| item.kind != UpdateChannelKind::DirectAsset)
    {
        let alternate = plan.update_strategy.alternates.remove(index);
        let previous = std::mem::replace(&mut plan.update_strategy.preferred, alternate);
        plan.update_strategy.alternates.insert(0, previous);
    }

    if let Some(canonical_locator) = plan.resolution.source.canonical_locator.clone() {
        plan.resolution.source.locator = canonical_locator;
        plan.resolution.source.normalized_kind = NormalizedSourceKind::GitHubRepository;
        plan.resolution.source.tracks_latest = true;
    }

    plan.interactions
        .retain(|interaction| interaction.key != "tracking-preference");
    plan
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AddPlan {
    pub resolution: AdapterResolution,
    pub selected_artifact: ArtifactCandidate,
    pub interactions: Vec<InteractionRequest>,
    pub update_strategy: UpdateStrategy,
    pub metadata: Vec<ParsedMetadata>,
}

pub fn materialize_app_record(
    source_input: &str,
    plan: &AddPlan,
) -> Result<AppRecord, MaterializeAddRecordError> {
    let identity_source = plan
        .resolution
        .source
        .canonical_locator
        .as_deref()
        .unwrap_or(source_input);
    let identity = resolve_identity(
        None,
        None,
        Some(identity_source),
        IdentityFallback::AllowRawUrl,
    )
    .map_err(MaterializeAddRecordError::Identity)?;

    Ok(AppRecord {
        stable_id: identity.stable_id,
        display_name: identity.display_name,
        source_input: Some(source_input.to_owned()),
        source: Some(plan.resolution.source.clone()),
        installed_version: Some(plan.selected_artifact.version.clone()),
        update_strategy: Some(plan.update_strategy.clone()),
        metadata: plan.metadata.clone(),
        install: None,
    })
}

pub fn install_app(
    source_input: &str,
    plan: &AddPlan,
    install_home: &Path,
    requested_scope: InstallScope,
) -> Result<InstalledApp, InstallAppError> {
    let mut reporter = NoopReporter;
    install_app_with_reporter(
        source_input,
        plan,
        install_home,
        requested_scope,
        &mut reporter,
    )
}

pub fn install_app_with_reporter(
    source_input: &str,
    plan: &AddPlan,
    install_home: &Path,
    requested_scope: InstallScope,
    reporter: &mut impl ProgressReporter,
) -> Result<InstalledApp, InstallAppError> {
    reporter.report(&OperationEvent::Started {
        kind: OperationKind::Add,
        label: source_input.to_owned(),
    });
    let mut record =
        materialize_app_record(source_input, plan).map_err(InstallAppError::Materialize)?;
    let (family, capabilities) =
        probe_live_host(install_home, requested_scope).map_err(InstallAppError::HostProbe)?;
    let policy = resolve_install_policy(family, requested_scope, &capabilities)
        .map_err(InstallAppError::Policy)?;
    let payload_path = resolve_target_path(
        install_home,
        &policy
            .payload_root
            .join(format!("{}.AppImage", record.stable_id)),
    );
    let desktop_path = resolve_target_path(
        install_home,
        &policy
            .desktop_entry_root
            .join(format!("aim-{}.desktop", record.stable_id)),
    );
    let icon_path = resolve_target_path(
        install_home,
        &policy.icon_root.join(format!("{}.png", record.stable_id)),
    );
    reporter.report(&OperationEvent::StageChanged {
        stage: OperationStage::DownloadArtifact,
        message: "downloading artifact".to_owned(),
    });
    let staging_root = install_home.join(".local/share/aim/staging");
    let staged_payload_path = staged_appimage_path(&staging_root, &record.stable_id);
    download_artifact_to_staged_path_with_reporter(
        &plan.selected_artifact.url,
        &staged_payload_path,
        reporter,
    )?;
    let payload_exec = payload_path.clone();
    let desktop_owned = match policy.integration_mode {
        IntegrationMode::PayloadOnly | IntegrationMode::Denied => None,
        IntegrationMode::Full | IntegrationMode::Degraded => Some((
            desktop_path.clone(),
            render_desktop_entry(&record.display_name, &payload_exec),
        )),
    };

    if desktop_owned.is_some() {
        reporter.report(&OperationEvent::StageChanged {
            stage: OperationStage::WriteDesktopEntry,
            message: "writing desktop entry".to_owned(),
        });
        reporter.report(&OperationEvent::StageChanged {
            stage: OperationStage::ExtractIcon,
            message: "extracting icon".to_owned(),
        });
    }

    reporter.report(&OperationEvent::StageChanged {
        stage: OperationStage::StagePayload,
        message: "staging payload".to_owned(),
    });
    let install_outcome = execute_install(&InstallRequest {
        staged_payload_path: &staged_payload_path,
        final_payload_path: &payload_path,
        trusted_checksum: plan.selected_artifact.trusted_checksum.as_deref(),
        desktop: desktop_owned.as_ref().map(|(path, contents)| {
            crate::integration::install::DesktopIntegrationRequest {
                desktop_entry_path: path.as_path(),
                desktop_entry_contents: contents.as_str(),
                icon_path: Some(icon_path.as_path()),
                icon_bytes: None,
            }
        }),
        helpers: capabilities.helpers.clone(),
    })
    .map_err(InstallAppError::Install)?;

    reporter.report(&OperationEvent::StageChanged {
        stage: OperationStage::RefreshIntegration,
        message: "refreshing desktop integration".to_owned(),
    });
    if !install_outcome.warnings.is_empty() {
        for warning in &install_outcome.warnings {
            reporter.report(&OperationEvent::Warning {
                message: warning.clone(),
            });
        }
    }

    record.install = Some(InstallMetadata {
        scope: policy.scope,
        payload_path: Some(install_outcome.final_payload_path.display().to_string()),
        desktop_entry_path: install_outcome
            .desktop_entry_path
            .as_ref()
            .map(|path| path.display().to_string()),
        icon_path: install_outcome
            .icon_path
            .as_ref()
            .map(|path| path.display().to_string()),
    });

    let installed = InstalledApp {
        record,
        selected_artifact: plan.selected_artifact.clone(),
        source: plan.resolution.source.clone(),
        install_scope: policy.scope,
        integration_mode: policy.integration_mode,
        install_outcome,
        warnings: policy.warnings,
    };

    reporter.report(&OperationEvent::Finished {
        summary: format!("installed {}", installed.record.stable_id),
    });

    Ok(installed)
}

#[derive(Debug, Eq, PartialEq)]
pub struct InstalledApp {
    pub record: AppRecord,
    pub selected_artifact: ArtifactCandidate,
    pub source: crate::domain::source::SourceRef,
    pub install_scope: InstallScope,
    pub integration_mode: IntegrationMode,
    pub install_outcome: InstallOutcome,
    pub warnings: Vec<String>,
}

#[derive(Debug)]
pub enum BuildAddPlanError {
    Query(ResolveQueryError),
    GitHubDiscovery(GitHubDiscoveryError),
    NoCandidates,
}

#[derive(Debug, Eq, PartialEq)]
pub enum MaterializeAddRecordError {
    Identity(ResolveIdentityError),
}

#[derive(Debug)]
pub enum InstallAppError {
    Materialize(MaterializeAddRecordError),
    Policy(String),
    Download(reqwest::Error),
    DownloadIo(std::io::Error),
    HostProbe(std::io::Error),
    Install(crate::integration::install::PayloadInstallError),
}

fn download_artifact_to_staged_path_with_reporter(
    url: &str,
    staged_payload_path: &Path,
    reporter: &mut impl ProgressReporter,
) -> Result<u64, InstallAppError> {
    let policy = http_client_policy();

    if env::var(FIXTURE_MODE_ENV).ok().as_deref() == Some("1") {
        let bytes = b"\x7fELFAppImage\x89PNG\r\n\x1a\nicondataIEND\xaeB`\x82";
        return download_to_staged_path_with_retries(staged_payload_path, reporter, policy, || {
            Ok((
                Box::new(std::io::Cursor::new(bytes.to_vec())) as Box<dyn Read>,
                Some(bytes.len() as u64),
            ))
        });
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(policy.timeout)
        .build()
        .map_err(InstallAppError::Download)?;

    download_to_staged_path_with_retries(staged_payload_path, reporter, policy, || {
        let response = client.get(url).send().map_err(InstallAppError::Download)?;
        let response = response
            .error_for_status()
            .map_err(InstallAppError::Download)?;
        let total = response.content_length();
        Ok((Box::new(response) as Box<dyn Read>, total))
    })
}

pub fn download_to_staged_path_with_retries(
    staged_payload_path: &Path,
    reporter: &mut impl ProgressReporter,
    policy: crate::source::github::HttpClientPolicy,
    mut open_stream: impl FnMut() -> Result<(Box<dyn Read>, Option<u64>), InstallAppError>,
) -> Result<u64, InstallAppError> {
    let mut last_error = None;
    let attempts = policy.max_retries.max(1);

    for attempt in 0..attempts {
        match open_stream() {
            Ok((mut reader, total)) => {
                match stream_payload_to_staged_file_with_reporter(
                    &mut reader,
                    total,
                    staged_payload_path,
                    reporter,
                ) {
                    Ok(written) => return Ok(written),
                    Err(error) if attempt + 1 < attempts && is_retryable_download_error(&error) => {
                        last_error = Some(error);
                    }
                    Err(error) => return Err(error),
                }
            }
            Err(error) if attempt + 1 < attempts && is_retryable_download_error(&error) => {
                last_error = Some(error);
            }
            Err(error) => return Err(error),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        InstallAppError::DownloadIo(std::io::Error::other("download failed after retries"))
    }))
}

pub fn stream_payload_to_staged_file_with_reporter<R: Read>(
    reader: &mut R,
    total: Option<u64>,
    staged_payload_path: &Path,
    reporter: &mut impl ProgressReporter,
) -> Result<u64, InstallAppError> {
    if let Some(parent) = staged_payload_path.parent() {
        fs::create_dir_all(parent).map_err(InstallAppError::DownloadIo)?;
    }

    let mut file = File::create(staged_payload_path).map_err(InstallAppError::DownloadIo)?;
    let mut buffer = [0_u8; 16 * 1024];
    let mut current = 0_u64;

    loop {
        let read = match reader.read(&mut buffer) {
            Ok(read) => read,
            Err(error) => {
                let _ = fs::remove_file(staged_payload_path);
                return Err(InstallAppError::DownloadIo(error));
            }
        };
        if read == 0 {
            break;
        }

        if let Err(error) = std::io::Write::write_all(&mut file, &buffer[..read]) {
            let _ = fs::remove_file(staged_payload_path);
            return Err(InstallAppError::DownloadIo(error));
        }
        current += read as u64;
        reporter.report(&OperationEvent::Progress { current, total });
    }

    Ok(current)
}

fn is_retryable_download_error(error: &InstallAppError) -> bool {
    matches!(
        error,
        InstallAppError::Download(_) | InstallAppError::DownloadIo(_)
    )
}

fn render_desktop_entry(display_name: &str, exec_path: &Path) -> String {
    format!(
        "[Desktop Entry]\nName={display_name}\nExec={}\nType=Application\nCategories=Utility;\n",
        exec_path.display()
    )
}

fn resolve_target_path(install_home: &Path, target: &Path) -> PathBuf {
    if target.is_absolute() {
        target.to_path_buf()
    } else {
        install_home.join(target)
    }
}

pub fn resolve_requested_scope(system: bool, user: bool, is_effective_root: bool) -> InstallScope {
    let override_scope = if system {
        Some(ScopeOverride::System)
    } else if user {
        Some(ScopeOverride::User)
    } else {
        None
    };

    resolve_install_scope_with_default(is_effective_root, override_scope)
}
