#[derive(Debug, Eq, PartialEq)]
pub struct UpdatePlan {
    pub items: Vec<PlannedUpdate>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PlannedUpdate {
    pub stable_id: String,
    pub display_name: String,
}
