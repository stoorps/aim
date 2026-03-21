use aim_core::app::add::AddPlan;
use aim_core::domain::search::SearchResults;
use aim_core::domain::show::{
    InstalledShow, MetadataSummary, RemoteInteractionSummary, RemoteShow, ShowResult, SourceSummary,
};
use aim_core::domain::update::UpdateExecutionStatus;
use console::measure_text_width;

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
        DispatchResult::Show(result) => render_show_result(result),
        DispatchResult::ShowAll(installed) => render_installed_show_list(installed),
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
        crate::ui::theme::heading(&format!(
            "Installed {} ({scope})",
            added.record.display_name
        )),
        format!(
            "{} {} {}",
            crate::ui::theme::label("Source"),
            added.source.kind.as_str(),
            added.source.locator,
        ),
        format!(
            "{} {}",
            crate::ui::theme::label("Artifact"),
            added.selected_artifact.url,
        ),
    ];

    let installed_files = install_file_paths(added);
    if !installed_files.is_empty() {
        lines.push(crate::ui::theme::label("Installed files"));
        lines.extend(
            installed_files
                .iter()
                .map(|path| crate::ui::theme::bullet(path)),
        );
    }

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

    let name_width = rows
        .iter()
        .map(|row| row.display_name.len())
        .max()
        .unwrap_or(0)
        .max("Name".len());
    let version_width = rows
        .iter()
        .map(|row| row.version.as_deref().unwrap_or("-").len())
        .max()
        .unwrap_or(0)
        .max("Version".len());

    let mut lines = vec![crate::ui::theme::heading("Installed Apps")];
    lines.push(format_list_row(
        "Name",
        "Version",
        "Source",
        name_width,
        version_width,
        true,
    ));

    for row in rows {
        lines.push(format_list_row(
            &row.display_name,
            row.version.as_deref().unwrap_or("-"),
            &row.source,
            name_width,
            version_width,
            false,
        ));
    }

    lines.join("\n")
}

fn format_list_row(
    name: &str,
    version: &str,
    source: &str,
    name_width: usize,
    version_width: usize,
    is_header: bool,
) -> String {
    let row = format!(
        "{name:<name_width$}  {version:<version_width$}  {source}",
        name = name,
        version = version,
        source = source,
        name_width = name_width,
        version_width = version_width,
    );

    if is_header {
        crate::ui::theme::label(&row)
    } else {
        row
    }
}

fn render_removed_app(removed: &aim_core::app::remove::RemovalResult) -> String {
    let warning_lines = removed
        .warnings
        .iter()
        .map(|warning| format!("Warning: {warning}"))
        .collect::<Vec<_>>();
    let mut lines = vec![crate::ui::theme::heading(&format!(
        "Removed {}",
        removed.removed.display_name,
    ))];

    if !removed.removed_paths.is_empty() {
        lines.push(crate::ui::theme::label("Removed files"));
        lines.extend(
            removed
                .removed_paths
                .iter()
                .map(|path| crate::ui::theme::bullet(path)),
        );
    }

    lines.extend(warning_lines);
    lines.join("\n")
}

fn render_show_result(result: &ShowResult) -> String {
    match result {
        ShowResult::Installed(installed) => render_installed_show(installed),
        ShowResult::Remote(remote) => render_remote_show(remote),
    }
}

fn render_installed_show_list(installed: &[InstalledShow]) -> String {
    if installed.is_empty() {
        return crate::ui::theme::muted("No installed apps yet");
    }

    installed
        .iter()
        .map(render_installed_show)
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_installed_show(installed: &InstalledShow) -> String {
    let mut lines = installed_title_lines(installed);

    if let Some(source_line) = installed_source_line(installed) {
        lines.push(source_line);
    }

    if let Some(source_input) = installed.source_input.as_deref()
        && should_render_requested_input(installed, source_input)
    {
        lines.push(format!(
            "{} {source_input}",
            crate::ui::theme::label("Requested")
        ));
    }

    if let Some(current_metadata) = installed.metadata.first() {
        lines.extend(metadata_detail_lines(current_metadata));
    }

    let tracked_paths = [
        installed.tracked_paths.payload_path.as_deref(),
        installed.tracked_paths.desktop_entry_path.as_deref(),
        installed.tracked_paths.icon_path.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if !tracked_paths.is_empty() {
        lines.push(installed_files_header(installed.install_scope));
        lines.extend(
            tracked_paths
                .into_iter()
                .map(|path| crate::ui::theme::muted(&format!("  {path}"))),
        );
    }

    lines.join("\n")
}

fn installed_title_lines(installed: &InstalledShow) -> Vec<String> {
    let left = crate::ui::theme::heading(&format!(
        "{} ({})",
        installed.display_name, installed.stable_id
    ));
    let right = installed_right_summary(installed);

    match terminal_width().filter(|width| *width > 0) {
        Some(width) => {
            let left_width = measure_text_width(&left);
            let right_width = measure_text_width(&right);
            if left_width + right_width + 2 <= width {
                vec![format!(
                    "{left}{}{right}",
                    " ".repeat(width - left_width - right_width)
                )]
            } else {
                vec![left, right]
            }
        }
        None => vec![left, right],
    }
}

fn installed_right_summary(installed: &InstalledShow) -> String {
    let mut parts = Vec::new();

    if let Some(version) = installed.installed_version.as_deref() {
        parts.push(crate::ui::theme::accent(&format!("v{version}")));
    }

    if let Some(tag) = installed_status_tag(installed) {
        parts.push(tag);
    }

    parts.join("  ")
}

fn installed_status_tag(installed: &InstalledShow) -> Option<String> {
    let versions = ordered_metadata_versions(&installed.metadata);
    let latest_version = versions.first()?.clone();
    let installed_version = installed.installed_version.as_deref()?;

    if installed_version == latest_version {
        Some(bold_muted("[up to date]"))
    } else {
        Some(crate::ui::theme::accent("[update available]"))
    }
}

fn installed_source_line(installed: &InstalledShow) -> Option<String> {
    let source = installed.source.as_ref()?;
    Some(labeled_detail_line(
        "Source",
        &format!(
            "{} - {}",
            source.kind.as_str(),
            display_source_locator(source)
        ),
    ))
}

fn display_source_locator(source: &SourceSummary) -> &str {
    source
        .canonical_locator
        .as_deref()
        .unwrap_or(source.locator.as_str())
}

fn should_render_requested_input(installed: &InstalledShow, source_input: &str) -> bool {
    let normalized_input = normalize_show_value(source_input);

    if normalized_input == normalize_show_value(&installed.display_name)
        || normalized_input == normalize_show_value(&installed.stable_id)
    {
        return false;
    }

    installed.source.as_ref().is_none_or(|source| {
        normalized_input != normalize_show_value(&source.locator)
            && source
                .canonical_locator
                .as_deref()
                .map(normalize_show_value)
                .is_none_or(|canonical| normalized_input != canonical)
    })
}

fn terminal_width() -> Option<usize> {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .or_else(|| {
            crossterm::terminal::size()
                .ok()
                .map(|(cols, _)| cols as usize)
        })
}

fn ordered_metadata_versions(metadata: &[MetadataSummary]) -> Vec<String> {
    let mut versions = Vec::new();

    for version in metadata.iter().filter_map(|item| item.version.as_deref()) {
        if !versions.iter().any(|existing| existing == version) {
            versions.push(version.to_owned());
        }
    }

    versions
}

fn metadata_detail_lines(metadata: &MetadataSummary) -> Vec<String> {
    let mut lines = vec![labeled_detail_line(
        "Update Mechanism",
        metadata_kind_label(metadata.kind),
    )];

    if let Some(architecture) = metadata.architecture.as_deref() {
        lines.push(labeled_detail_line("Architecture", architecture));
    }

    if let Some(checksum) = metadata.checksum.as_deref() {
        lines.push(labeled_detail_line(
            "Checksum",
            &truncate_checksum(checksum),
        ));
    }

    lines
}

fn installed_files_header(scope: Option<aim_core::domain::app::InstallScope>) -> String {
    let label = match scope {
        Some(aim_core::domain::app::InstallScope::User) => "Installed as User",
        Some(aim_core::domain::app::InstallScope::System) => "Installed as System",
        None => "Installed files",
    };

    bold_muted_label(label)
}

fn labeled_detail_line(label: &str, value: &str) -> String {
    format!(
        "{} {}",
        bold_muted_label(label),
        crate::ui::theme::muted(value)
    )
}

fn truncate_checksum(checksum: &str) -> String {
    const PREFIX_CHARS: usize = 14;
    const SUFFIX_CHARS: usize = 6;
    const ELLIPSIS_CHARS: usize = 3;

    let checksum_len = checksum.chars().count();

    if checksum_len <= PREFIX_CHARS + SUFFIX_CHARS + ELLIPSIS_CHARS {
        checksum.to_owned()
    } else {
        let prefix = checksum.chars().take(PREFIX_CHARS).collect::<String>();
        let suffix = checksum
            .chars()
            .skip(checksum_len - SUFFIX_CHARS)
            .collect::<String>();
        format!("{prefix}...{suffix}",)
    }
}

fn metadata_kind_label(kind: aim_core::domain::update::ParsedMetadataKind) -> &'static str {
    match kind {
        aim_core::domain::update::ParsedMetadataKind::Unknown => "unknown",
        aim_core::domain::update::ParsedMetadataKind::ElectronBuilder => "electron-builder",
        aim_core::domain::update::ParsedMetadataKind::Zsync => "zsync",
    }
}

fn normalize_show_value(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn bold_muted(message: &str) -> String {
    let mut style = crate::ui::theme::current_theme().muted;
    style.bold = true;
    crate::ui::theme::apply_style_spec(message, &style)
}

fn bold_muted_label(label: &str) -> String {
    bold_muted(&format!("{label}:"))
}

fn render_remote_show(remote: &RemoteShow) -> String {
    let mut lines = vec![crate::ui::theme::heading("Resolved Source")];
    lines.push(format!(
        "{} {} {}",
        crate::ui::theme::label("Source"),
        remote.source.kind.as_str(),
        remote.source.locator,
    ));
    if let Some(canonical_locator) = remote.source.canonical_locator.as_deref() {
        lines.push(format!(
            "{} {canonical_locator}",
            crate::ui::theme::label("Canonical")
        ));
    }
    lines.push(format!(
        "{} {}",
        crate::ui::theme::label("Artifact"),
        remote.artifact.url,
    ));
    if let Some(version) = remote.artifact.version.as_deref() {
        lines.push(format!("{} {version}", crate::ui::theme::label("Version")));
    }
    if let Some(checksum) = remote.artifact.trusted_checksum.as_deref() {
        lines.push(format!(
            "{} {checksum}",
            crate::ui::theme::label("Checksum")
        ));
    }
    lines.push(format!(
        "{} {}",
        crate::ui::theme::label("Selection"),
        remote.artifact.selection_reason,
    ));

    if !remote.interactions.is_empty() {
        lines.push(crate::ui::theme::label("Interactions"));
        for interaction in &remote.interactions {
            let text = match interaction {
                RemoteInteractionSummary::ChooseTrackingPreference {
                    requested_version,
                    latest_version,
                } => format!(
                    "choose tracking preference: requested {requested_version}, latest {latest_version}"
                ),
                RemoteInteractionSummary::SelectArtifact { candidate_count } => {
                    format!("select artifact: {candidate_count} candidates")
                }
            };
            lines.push(crate::ui::theme::bullet(&text));
        }
    }

    if !remote.warnings.is_empty() {
        lines.push(crate::ui::theme::label("Warnings"));
        lines.extend(
            remote
                .warnings
                .iter()
                .map(|warning| format!("Warning: {warning}")),
        );
    }

    lines.join("\n")
}

fn install_file_paths(added: &aim_core::app::add::InstalledApp) -> Vec<String> {
    [
        Some(
            added
                .install_outcome
                .final_payload_path
                .display()
                .to_string(),
        ),
        added
            .install_outcome
            .desktop_entry_path
            .as_ref()
            .map(|path| path.display().to_string()),
        added
            .install_outcome
            .icon_path
            .as_ref()
            .map(|path| path.display().to_string()),
    ]
    .into_iter()
    .flatten()
    .collect()
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
