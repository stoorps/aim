use aim_cli::config::SearchConfig;
use aim_cli::ui::search_browser::{BrowserPhase, SearchBrowserState, SubmitAction};
use aim_core::domain::search::{SearchInstallStatus, SearchResult};

#[test]
fn browser_defaults_to_bottom_to_top_ordering() {
    let state = SearchBrowserState::new(sample_results(), SearchConfig::default(), 3);

    assert_eq!(
        visible_names(&state),
        vec!["charlie/app", "bravo/app", "alpha/app"]
    );
}

#[test]
fn browser_moves_cursor_and_pages() {
    let mut state = SearchBrowserState::new(sample_results(), SearchConfig::default(), 2);

    state.move_next();
    assert_eq!(state.cursor_position(), 1);

    state.page_down();
    assert_eq!(state.cursor_position(), 2);

    state.page_up();
    assert_eq!(state.cursor_position(), 0);
}

#[test]
fn browser_supports_single_and_multiple_numeric_selection() {
    let mut state = SearchBrowserState::new(sample_results(), SearchConfig::default(), 3);

    state.apply_numeric_selection("1,3").unwrap();

    assert_eq!(selected_names(&state), vec!["charlie/app", "alpha/app"]);
}

#[test]
fn browser_supports_numeric_ranges() {
    let mut state = SearchBrowserState::new(sample_results(), SearchConfig::default(), 3);

    state.apply_numeric_selection("1-2").unwrap();

    assert_eq!(selected_names(&state), vec!["charlie/app", "bravo/app"]);
}

#[test]
fn browser_supports_space_separated_numeric_selection() {
    let mut state = SearchBrowserState::new(sample_results(), SearchConfig::default(), 3);

    state.apply_numeric_selection("1 3").unwrap();

    assert_eq!(selected_names(&state), vec!["charlie/app", "alpha/app"]);
}

#[test]
fn typing_numeric_input_updates_selection_immediately() {
    let mut state = SearchBrowserState::new(sample_results(), SearchConfig::default(), 3);

    state.push_numeric_input('1');
    assert_eq!(selected_names(&state), vec!["charlie/app"]);

    state.push_numeric_input(' ');
    state.push_numeric_input('3');

    assert_eq!(selected_names(&state), vec!["charlie/app", "alpha/app"]);
}

#[test]
fn invalid_numeric_input_keeps_last_good_selection_visible() {
    let mut state = SearchBrowserState::new(sample_results(), SearchConfig::default(), 3);

    state.push_numeric_input('1');
    assert_eq!(selected_names(&state), vec!["charlie/app"]);

    state.push_numeric_input('-');

    assert_eq!(selected_names(&state), vec!["charlie/app"]);
    assert_eq!(state.numeric_buffer(), "1-");
}

#[test]
fn highlight_segments_marks_matching_query_fragments() {
    let fragments = aim_cli::ui::search_browser::highlight_segments("pingdotgg/t3code", "dotgg");

    assert_eq!(fragments.len(), 3);
    assert_eq!(fragments[1].text, "dotgg");
    assert!(fragments[1].is_match);
}

#[test]
fn invalid_numeric_selection_preserves_existing_selection() {
    let mut state = SearchBrowserState::new(sample_results(), SearchConfig::default(), 3);
    state.apply_numeric_selection("2").unwrap();

    let error = state.apply_numeric_selection("2-z").unwrap_err();

    assert!(error.contains("2-z"));
    assert_eq!(selected_names(&state), vec!["bravo/app"]);
}

#[test]
fn confirmation_requires_selection_before_transition() {
    let mut state = SearchBrowserState::new(sample_results(), SearchConfig::default(), 3);

    assert!(!state.enter_confirmation());
    assert_eq!(state.phase(), BrowserPhase::Browsing);

    state.toggle_current_selection();
    assert!(state.enter_confirmation());
    assert_eq!(state.phase(), BrowserPhase::Confirming);

    state.cancel_confirmation();
    assert_eq!(state.phase(), BrowserPhase::Browsing);
}

#[test]
fn submit_selection_can_skip_confirmation_from_config() {
    let mut state = SearchBrowserState::new(sample_results(), SearchConfig::default(), 3);
    state.toggle_current_selection();

    let action = state.submit_selection(true);

    assert_eq!(
        action,
        SubmitAction::Confirmed(aim_cli::ui::search_browser::SearchSelection {
            rows: vec![aim_cli::ui::search_browser::SearchRow {
                status: SearchInstallStatus::Available,
                provider_id: "github".to_owned(),
                display_name: "charlie/app".to_owned(),
                description: None,
                install_query: "charlie/app".to_owned(),
                version: Some("1.0.0".to_owned()),
                selectable: true,
            }],
        })
    );
}

#[test]
fn installed_rows_are_visible_but_not_selectable() {
    let mut state = SearchBrowserState::new(installed_first_results(), SearchConfig::default(), 3);

    state.toggle_current_selection();

    assert!(state.selected_rows().is_empty());
    assert_eq!(
        state.status_message(),
        Some("installed result is not selectable")
    );
}

#[test]
fn update_rows_remain_selectable() {
    let mut state = SearchBrowserState::new(update_first_results(), SearchConfig::default(), 3);

    state.toggle_current_selection();

    assert_eq!(selected_names(&state), vec!["charlie/app"]);
}

#[test]
fn selection_expression_prefills_from_checklist_selection() {
    let mut state = SearchBrowserState::new(sample_results(), SearchConfig::default(), 5);

    state.toggle_current_selection();
    state.move_to_bottom();
    state.toggle_current_selection();

    assert_eq!(state.selection_expression(), "1,3");
}

#[test]
fn selection_expression_compacts_adjacent_ranges() {
    let mut state = SearchBrowserState::new(sample_results(), SearchConfig::default(), 5);

    state.apply_numeric_selection("1-3").unwrap();

    assert_eq!(state.selection_expression(), "1-3");
}

fn sample_results() -> Vec<SearchResult> {
    vec![
        sample_result("alpha/app"),
        sample_result("bravo/app"),
        sample_result("charlie/app"),
    ]
}

fn sample_result(name: &str) -> SearchResult {
    SearchResult {
        provider_id: "github".to_owned(),
        display_name: name.to_owned(),
        description: None,
        source_locator: name.to_owned(),
        install_query: name.to_owned(),
        canonical_locator: name.to_owned(),
        version: Some("1.0.0".to_owned()),
        install_status: SearchInstallStatus::Available,
    }
}

fn installed_first_results() -> Vec<SearchResult> {
    let mut results = sample_results();
    results[2].install_status = SearchInstallStatus::Installed {
        installed_version: Some("1.0.0".to_owned()),
    };
    results
}

fn update_first_results() -> Vec<SearchResult> {
    let mut results = sample_results();
    results[2].install_status = SearchInstallStatus::UpdateAvailable {
        installed_version: Some("0.9.0".to_owned()),
        latest_version: Some("1.0.0".to_owned()),
    };
    results
}

fn visible_names(state: &SearchBrowserState) -> Vec<&str> {
    state
        .ordered_rows()
        .iter()
        .map(|row| row.display_name.as_str())
        .collect()
}

fn selected_names(state: &SearchBrowserState) -> Vec<&str> {
    state
        .selected_rows()
        .iter()
        .map(|row| row.display_name.as_str())
        .collect()
}
