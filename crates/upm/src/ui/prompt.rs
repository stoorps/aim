use std::env;
use std::io::IsTerminal;

use dialoguer::Select;
use upm_core::app::add::{AddPlan, prefer_latest_tracking};
use upm_core::app::interaction::{InteractionKind, InteractionRequest};

const TRACKING_PREFERENCE_ENV: &str = "UPM_TRACKING_PREFERENCE";

pub fn render_interaction(request: &InteractionRequest) -> String {
    match &request.kind {
        InteractionKind::SelectRegisteredApp { query, matches } => format!(
            "Choose the installed app matching '{query}': {}",
            matches.join(", ")
        ),
        InteractionKind::ChooseTrackingPreference {
            requested_version,
            latest_version,
        } => format!(
            "Choose update tracking: requested {requested_version}, latest available {latest_version}",
        ),
        InteractionKind::SelectArtifact { candidates } => {
            format!("Choose an artifact: {}", candidates.join(", "))
        }
    }
}

pub fn render_interactions(requests: &[InteractionRequest]) -> String {
    requests
        .iter()
        .map(render_interaction)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn resolve_add_plan_interactions(plan: AddPlan) -> Result<Option<AddPlan>, PromptError> {
    let mut resolved = plan;

    for request in resolved.interactions.clone() {
        match &request.kind {
            InteractionKind::ChooseTrackingPreference {
                requested_version,
                latest_version,
            } => match resolve_tracking_preference(requested_version, latest_version)? {
                Some(TrackingPreference::Requested) => {
                    resolved
                        .interactions
                        .retain(|item| item.key != "tracking-preference");
                }
                Some(TrackingPreference::Latest) => {
                    resolved = prefer_latest_tracking(resolved);
                }
                None => return Ok(None),
            },
            InteractionKind::SelectRegisteredApp { .. }
            | InteractionKind::SelectArtifact { .. } => {
                return Ok(None);
            }
        }
    }

    Ok(Some(resolved))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TrackingPreference {
    Requested,
    Latest,
}

#[derive(Debug)]
pub enum PromptError {
    InvalidTrackingPreference(String),
    Dialoguer(dialoguer::Error),
}

impl From<dialoguer::Error> for PromptError {
    fn from(value: dialoguer::Error) -> Self {
        Self::Dialoguer(value)
    }
}

fn resolve_tracking_preference(
    requested_version: &str,
    latest_version: &str,
) -> Result<Option<TrackingPreference>, PromptError> {
    if let Ok(value) = env::var(TRACKING_PREFERENCE_ENV) {
        return match value.trim().to_ascii_lowercase().as_str() {
            "requested" | "current" => Ok(Some(TrackingPreference::Requested)),
            "latest" => Ok(Some(TrackingPreference::Latest)),
            other => Err(PromptError::InvalidTrackingPreference(other.to_owned())),
        };
    }

    if !std::io::stdin().is_terminal() {
        return Ok(None);
    }

    let options = [
        format!("Keep tracking the requested release lineage ({requested_version})"),
        format!("Track the latest release after install ({latest_version})"),
    ];
    let selection = Select::with_theme(&crate::ui::theme::dialog_theme())
        .with_prompt("Choose update tracking")
        .items(options)
        .default(1)
        .interact()?;

    Ok(Some(match selection {
        0 => TrackingPreference::Requested,
        _ => TrackingPreference::Latest,
    }))
}
