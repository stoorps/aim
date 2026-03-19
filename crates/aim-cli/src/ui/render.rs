use aim_core::domain::source::SourceRef;

use crate::DispatchResult;

pub fn render_update_summary(total: usize, selected: usize, failed: usize) -> String {
    format!("updates found: {total}, selected: {selected}, failed: {failed}",)
}

pub fn render_dispatch_result(result: &DispatchResult) -> String {
    match result {
        DispatchResult::AddPlan(source) => render_add_plan(source),
        DispatchResult::List(rows) => render_list(rows),
        DispatchResult::Removed(display_name) => format!("removed: {display_name}"),
        DispatchResult::UpdatePlan(plan) => {
            render_update_summary(plan.items.len(), plan.items.len(), 0)
        }
        DispatchResult::Noop => String::new(),
    }
}

fn render_add_plan(source: &SourceRef) -> String {
    format!(
        "resolved source: {} {}",
        source.kind.as_str(),
        source.locator
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
