use crate::domain::app::AppRecord;
use crate::domain::update::{PlannedUpdate, UpdatePlan};

pub fn build_update_plan(apps: &[AppRecord]) -> Result<UpdatePlan, BuildUpdatePlanError> {
    Ok(UpdatePlan {
        items: apps
            .iter()
            .map(|app| PlannedUpdate {
                stable_id: app.stable_id.clone(),
                display_name: app.display_name.clone(),
            })
            .collect(),
    })
}

#[derive(Debug, Eq, PartialEq)]
pub enum BuildUpdatePlanError {}
