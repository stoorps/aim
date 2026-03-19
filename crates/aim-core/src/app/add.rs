use crate::adapters::traits::AdapterResolution;
use crate::app::identity::{IdentityFallback, ResolveIdentityError, resolve_identity};
use crate::app::interaction::{InteractionKind, InteractionRequest};
use crate::app::query::{ResolveQueryError, resolve_query};
use crate::domain::app::AppRecord;
use crate::domain::source::{NormalizedSourceKind, ResolvedRelease, SourceKind};
use crate::domain::update::{ArtifactCandidate, ParsedMetadata, UpdateChannelKind, UpdateStrategy};
use crate::metadata::parse_document;
use crate::source::github::{
    GitHubDiscoveryError, GitHubTransport, discover_github_candidates_with,
};
use crate::update::channels::build_channels;
use crate::update::ranking::{rank_channels, select_artifact, to_preference};

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
    })
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
