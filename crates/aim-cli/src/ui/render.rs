use aim_core::app::add::AddPlan;
use aim_core::domain::search::SearchResults;
use aim_core::domain::update::UpdateExecutionStatus;

use crate::DispatchResult;
use crate::config::CliConfig;

pub fn render_update_summary(total: usize, selected: usize, failed: usize) -> String {
    [
        crate::ui::theme::heading("Update Review"),
        format!("apps with updates: {total}"),
        format!("selected: {selected}"),
        format!("failed: {failed}"),
    ]
    .join("\n")
}

pub fn render_dispatch_result(result: &DispatchResult) -> String {
    render_dispatch_result_with_config(result, &CliConfig::default())
}

pub fn render_dispatch_result_with_config(result: &DispatchResult, config: &CliConfig) -> String {
    match result {
        DispatchResult::Added(added) => render_added_app(added),
        DispatchResult::List(rows) => render_list(rows),
        DispatchResult::PendingAdd(plan) => render_pending_add(plan),
        DispatchResult::Removed(removed) => render_removed_app(removed),
        DispatchResult::Search(results) => render_search_results_with_config(results, config),
        DispatchResult::UpdatePlan(plan) => render_update_plan(plan),
        DispatchResult::Updated(result) => render_updated_apps(result),
        DispatchResult::Noop => String::new(),
    }
}

fn render_added_app(added: &aim_core::app::add::InstalledApp) -> String {
    let scope = match added.install_scope {
        aim_core::domain::app::InstallScope::User => "user",
        aim_core::domain::app::InstallScope::System => "system",
    };

    let warning_lines = added
        .warnings
        .iter()
        .chain(added.install_outcome.warnings.iter())
        .map(|warning| format!("Warning: {warning}"))
        .collect::<Vec<_>>();

    let mut lines = vec![
        crate::ui::theme::heading("Installation Summary"),
        format!(
            "{} {} ({})",
            crate::ui::theme::label("Application"),
            added.record.display_name,
            added.record.stable_id,
        ),
        format!("{} {scope}", crate::ui::theme::label("Install scope")),
        format!(
            "{} {} {}",
            crate::ui::theme::label("Source"),
            added.source.kind.as_str(),
            added.source.locator,
        ),
        format!(
            "{} {} [{}]",
            crate::ui::theme::label("Selected artifact"),
            added.selected_artifact.url,
            added.selected_artifact.selection_reason,
        ),
    ];

    lines.extend(warning_lines);
    lines.join("\n")
}

fn render_pending_add(plan: &AddPlan) -> String {
    let prompts = crate::ui::prompt::render_interactions(&plan.interactions);
    [
        crate::ui::theme::heading("Installation Review"),
        format!(
            "{} {} {}",
            crate::ui::theme::label("Resolved source"),
            plan.resolution.source.kind.as_str(),
            plan.resolution.source.locator,
        ),
        format!(
            "{} {} [{}]",
            crate::ui::theme::label("Selected artifact"),
            plan.selected_artifact.url,
            plan.selected_artifact.selection_reason,
        ),
        prompts,
    ]
    .join("\n")
}

fn render_list(rows: &[aim_core::app::list::ListRow]) -> String {
    if rows.is_empty() {
        return crate::ui::theme::muted("No installed apps yet");
    }

    let mut output = format!("{}\n", crate::ui::theme::heading("Installed Apps"));
    for row in rows {
        output.push_str(&format!(
            "{}\n",
            crate::ui::theme::bullet(&format!("{} ({})", row.display_name, row.stable_id))
        ));
    }
    output.trim_end().to_owned()
}

fn render_removed_app(removed: &aim_core::app::remove::RemovalResult) -> String {
    let warning_lines = removed
        .warnings
        .iter()
        .map(|warning| format!("Warning: {warning}"))
        .collect::<Vec<_>>();
    let mut lines = vec![
        crate::ui::theme::heading("Removal Summary"),
        format!(
            "{} {}",
            crate::ui::theme::label("Removed app"),
            removed.removed.display_name,
        ),
    ];
    lines.extend(warning_lines);
    lines.join("\n")
}

fn render_search_results(results: &SearchResults) -> String {
    let mut lines = vec![crate::ui::theme::heading("Search Results")];

    lines.push(crate::ui::theme::heading("Remote Results"));
    if results.remote_hits.is_empty() {
        lines.push(crate::ui::theme::muted("No remote matches"));
    } else {
        for hit in &results.remote_hits {
            lines.push(crate::ui::theme::bullet(&format!(
                "[{}] {}",
                hit.provider_id, hit.display_name
            )));
            lines.push(format!("Install query: {}", hit.install_query));
            lines.push(format!("Source: {}", hit.source_locator));
            if let Some(description) = &hit.description {
                lines.push(format!("Description: {description}"));
            }
        }
    }

    lines.push(crate::ui::theme::heading("Installed Matches"));
    if results.installed_matches.is_empty() {
        lines.push(crate::ui::theme::muted("No installed matches"));
    } else {
        for app in &results.installed_matches {
            lines.push(crate::ui::theme::bullet(&format!(
                "{} ({})",
                app.display_name, app.stable_id
            )));
        }
    }

    if !results.warnings.is_empty() {
        lines.push(crate::ui::theme::heading("Warnings"));
        for warning in &results.warnings {
            match warning.provider_id.as_deref() {
                Some(provider_id) => {
                    lines.push(format!("Warning: {provider_id}: {}", warning.message))
                }
                None => lines.push(format!("Warning: {}", warning.message)),
            }
        }
    }

    lines.join("\n")
}

fn render_search_results_with_config(results: &SearchResults, config: &CliConfig) -> String {
    if crate::ui::search_browser::can_launch(results) {
        match crate::ui::search_browser::run(results, config) {
            Ok(Some(selection)) => {
                return crate::ui::search_browser::render_confirmation_summary(&selection.rows);
            }
            Ok(None) => return String::new(),
            Err(_) => {}
        }
    }

    render_search_results(results)
}

fn render_updated_apps(result: &aim_core::domain::update::UpdateExecutionResult) -> String {
    let mut lines = vec![
        crate::ui::theme::heading("Update Summary"),
        format!("updated apps: {}", result.updated_count()),
        format!("failed updates: {}", result.failed_count()),
    ];

    for item in &result.items {
        match &item.status {
            UpdateExecutionStatus::Updated => lines.push(format!(
                "Updated: {} ({}) {} -> {}",
                item.display_name,
                item.stable_id,
                item.from_version.as_deref().unwrap_or("unknown"),
                item.to_version.as_deref().unwrap_or("unknown")
            )),
            UpdateExecutionStatus::Failed { reason } => lines.push(format!(
                "Failed: {} ({}) {}",
                item.display_name, item.stable_id, reason
            )),
        }
    }

    lines.join("\n")
}

fn render_update_plan(plan: &aim_core::domain::update::UpdatePlan) -> String {
    let mut lines = vec![render_update_summary(plan.items.len(), plan.items.len(), 0)];

    for item in &plan.items {
        lines.push(format!(
            "{} ({}) via {}",
            item.display_name, item.stable_id, item.selected_channel.locator
        ));
    }

    lines.join("\n")
}
