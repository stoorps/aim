use aim_core::app::update::build_update_plan;
use aim_core::domain::app::AppRecord;

#[test]
fn empty_registry_produces_empty_plan() {
    let plan = build_update_plan(&[]).unwrap();

    assert!(plan.items.is_empty());
}

#[test]
fn installed_apps_are_carried_into_review_plan() {
    let apps = [AppRecord {
        stable_id: "bat".to_owned(),
        display_name: "Bat".to_owned(),
    }];

    let plan = build_update_plan(&apps).unwrap();

    assert_eq!(plan.items.len(), 1);
    assert_eq!(plan.items[0].stable_id, "bat");
}
