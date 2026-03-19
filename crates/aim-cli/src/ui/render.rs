use aim_core::app::add::AddPlan;

use crate::DispatchResult;

pub fn render_update_summary(total: usize, selected: usize, failed: usize) -> String {
    format!("updates found: {total}, selected: {selected}, failed: {failed}",)
}

pub fn render_dispatch_result(result: &DispatchResult) -> String {
    match result {
        DispatchResult::Added(added) => render_added_app(added),
        DispatchResult::List(rows) => render_list(rows),
        DispatchResult::PendingAdd(plan) => render_pending_add(plan),
        DispatchResult::Removed(display_name) => format!("removed: {display_name}"),
        DispatchResult::UpdatePlan(plan) => {
            render_update_summary(plan.items.len(), plan.items.len(), 0)
        }
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
        .map(|warning| format!("warning: {warning}"))
        .collect::<Vec<_>>()
        .join("\n");

    let summary = format!(
        "installing as {scope}\ninstalled app: {} ({})\nsource: {} {}\nselected artifact: {} [{}]",
        added.record.display_name,
        added.record.stable_id,
        added.source.kind.as_str(),
        added.source.locator,
        added.selected_artifact.url,
        added.selected_artifact.selection_reason,
    );

    if warning_lines.is_empty() {
        summary
    } else {
        format!("{summary}\n{warning_lines}")
    }
}

fn render_pending_add(plan: &AddPlan) -> String {
    let prompts = crate::ui::prompt::render_interactions(&plan.interactions);
    format!(
        "resolved source: {} {}\nselected artifact: {} [{}]\n{prompts}",
        plan.resolution.source.kind.as_str(),
        plan.resolution.source.locator,
        plan.selected_artifact.url,
        plan.selected_artifact.selection_reason,
    )
}

fn render_list(rows: &[aim_core::app::list::ListRow]) -> String {
    if rows.is_empty() {
        return "installed apps: none".to_owned();
    }

    let mut output = String::from("installed apps:\n");
    for row in rows {
        output.push_str(&format!("- {} ({})\n", row.display_name, row.stable_id));
    }
    output.trim_end().to_owned()
}
