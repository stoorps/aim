use aim_core::app::interaction::{InteractionKind, InteractionRequest};
use aim_core::app::list::build_list_rows;
use aim_core::app::remove::resolve_registered_app;
use aim_core::domain::app::AppRecord;

#[test]
fn remove_flow_rejects_unknown_app_names() {
    let result = resolve_registered_app("bat", &[]);

    assert!(result.is_err());
}

#[test]
fn list_flow_returns_display_rows_for_registered_apps() {
    let rows = build_list_rows(&[AppRecord {
        stable_id: "bat".to_owned(),
        display_name: "Bat".to_owned(),
        source_input: None,
        source: None,
        installed_version: None,
        update_strategy: None,
        metadata: Vec::new(),
    }]);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].stable_id, "bat");
    assert_eq!(rows[0].display_name, "Bat");
}

#[test]
fn ambiguous_remove_matches_include_stable_ids_for_client_choice() {
    let apps = [
        AppRecord {
            stable_id: "bat".to_owned(),
            display_name: "Bat".to_owned(),
            source_input: None,
            source: None,
            installed_version: None,
            update_strategy: None,
            metadata: Vec::new(),
        },
        AppRecord {
            stable_id: "bat-nightly".to_owned(),
            display_name: "Bat".to_owned(),
            source_input: None,
            source: None,
            installed_version: None,
            update_strategy: None,
            metadata: Vec::new(),
        },
    ];

    let error = resolve_registered_app("Bat", &apps).unwrap_err();

    assert_eq!(
        error,
        aim_core::app::remove::ResolveRegisteredAppError::Ambiguous {
            request: InteractionRequest {
                key: "select-registered-app".to_owned(),
                kind: InteractionKind::SelectRegisteredApp {
                    query: "Bat".to_owned(),
                    matches: vec!["Bat (bat)".to_owned(), "Bat (bat-nightly)".to_owned()],
                },
            },
        }
    );
}
