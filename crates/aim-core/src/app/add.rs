use crate::adapters::github::{GitHubAdapter, GitHubAdapterError};
use crate::adapters::traits::AdapterResolution;
use crate::app::query::{ResolveQueryError, resolve_query};
use crate::domain::source::{SourceKind, SourceRef};

pub fn build_add_plan(query: &str) -> Result<AddPlan, BuildAddPlanError> {
    let source = resolve_query(query).map_err(BuildAddPlanError::Query)?;

    let resolution = match source.kind {
        SourceKind::GitHub => GitHubAdapter::new()
            .resolve(&source)
            .map_err(BuildAddPlanError::GitHub)?,
        _ => AdapterResolution {
            source: SourceRef {
                kind: source.kind,
                locator: source.locator.clone(),
            },
            release: crate::domain::source::ResolvedRelease {
                version: "unresolved".to_owned(),
            },
        },
    };

    Ok(AddPlan { resolution })
}

#[derive(Debug, Eq, PartialEq)]
pub struct AddPlan {
    pub resolution: AdapterResolution,
}

#[derive(Debug, Eq, PartialEq)]
pub enum BuildAddPlanError {
    Query(ResolveQueryError),
    GitHub(GitHubAdapterError),
}
