use crate::domain::app::AppRecord;
use crate::domain::update::{ChannelPreference, PlannedUpdate, UpdateChannelKind, UpdatePlan};

pub fn build_update_plan(apps: &[AppRecord]) -> Result<UpdatePlan, BuildUpdatePlanError> {
    Ok(UpdatePlan {
        items: apps.iter().map(plan_update).collect(),
    })
}

#[derive(Debug, Eq, PartialEq)]
pub enum BuildUpdatePlanError {}

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
