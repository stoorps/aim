use std::io::IsTerminal;
use std::time::Duration;

use aim_core::app::progress::{OperationEvent, OperationKind, OperationStage, ProgressReporter};
use indicatif::{ProgressBar, ProgressStyle};

pub fn new_progress_bar(total: Option<u64>) -> ProgressBar {
    match total {
        Some(total) => ProgressBar::new(total),
        None => ProgressBar::new_spinner(),
    }
}

pub fn spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("{spinner} {msg}").expect("spinner template is valid")
}

pub fn byte_style() -> ProgressStyle {
    let theme = crate::ui::theme::current_theme();
    let filled = crate::ui::theme::indicatif_color_key(&theme.progress_bar);
    let unfilled = crate::ui::theme::indicatif_color_key(&theme.progress_bar_unfilled);
    ProgressStyle::with_template(&format!(
        "{{bar:32.{filled}/{unfilled}}} {{bytes}}/{{total_bytes}} {{msg}}"
    ))
    .expect("byte progress template is valid")
}

pub fn operation_label(kind: OperationKind) -> &'static str {
    match kind {
        OperationKind::Add => "Installing",
        OperationKind::Search => "Searching",
        OperationKind::UpdateBatch => "Updating",
        OperationKind::UpdateItem => "Updating",
        OperationKind::Remove => "Removing",
    }
}

pub fn stage_label(stage: OperationStage) -> &'static str {
    match stage {
        OperationStage::ResolveQuery => "Resolving source",
        OperationStage::DiscoverRelease => "Discovering release",
        OperationStage::SelectArtifact => "Selecting artifact",
        OperationStage::DownloadArtifact => "Downloading artifact",
        OperationStage::StagePayload => "Staging payload",
        OperationStage::WriteDesktopEntry => "Writing desktop entry",
        OperationStage::ExtractIcon => "Extracting icon",
        OperationStage::RefreshIntegration => "Refreshing desktop integration",
        OperationStage::SaveRegistry => "Saving registry",
        OperationStage::Finalize => "Finalizing",
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut value = bytes as f64;
    let mut unit_index = 0_usize;
    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{value:.1} {}", UNITS[unit_index])
    }
}

pub fn format_completed_stage_line(token: &str) -> String {
    format!("{} {token}", crate::ui::theme::success("✓"))
}

pub fn event_message(event: &OperationEvent) -> Option<String> {
    match event {
        OperationEvent::Started { kind, label } => {
            Some(format!("{} {label}", operation_label(*kind)))
        }
        OperationEvent::StageChanged { stage, message } => {
            let title = stage_label(*stage);
            if title.eq_ignore_ascii_case(message) {
                Some(title.to_owned())
            } else {
                Some(format!("{title}: {message}"))
            }
        }
        OperationEvent::Progress { .. } => None,
        OperationEvent::Warning { message } => Some(format!("Warning: {message}")),
        OperationEvent::Finished { summary } => Some(summary.clone()),
        OperationEvent::Failed { stage, reason } => {
            Some(format!("{} failed: {reason}", stage_label(*stage)))
        }
    }
}

pub struct TerminalProgressReporter {
    interactive: bool,
    progress_bar: Option<ProgressBar>,
    byte_total: Option<u64>,
    current_stage: Option<OperationStage>,
    last_progress_bytes: Option<u64>,
    emitted_output: bool,
}

impl TerminalProgressReporter {
    pub fn stderr() -> Self {
        Self {
            interactive: std::io::stderr().is_terminal(),
            progress_bar: None,
            byte_total: None,
            current_stage: None,
            last_progress_bytes: None,
            emitted_output: false,
        }
    }

    pub fn emitted_output(&self) -> bool {
        self.emitted_output
    }

    fn clear_progress(&mut self) {
        if let Some(progress_bar) = self.progress_bar.take() {
            progress_bar.finish_and_clear();
        }
        self.byte_total = None;
    }

    fn emit_completed_stage_token(&mut self) {
        let token = match self.current_stage {
            Some(OperationStage::DownloadArtifact) => self
                .last_progress_bytes
                .map(|bytes| format!("{} Downloaded", format_bytes(bytes))),
            Some(OperationStage::StagePayload) => Some("Payload Staged".to_owned()),
            Some(OperationStage::WriteDesktopEntry) => Some("Desktop Entry Written".to_owned()),
            Some(OperationStage::ExtractIcon) => Some("Icon Extracted".to_owned()),
            Some(OperationStage::RefreshIntegration) => {
                Some("Desktop Integration Refreshed".to_owned())
            }
            Some(OperationStage::SaveRegistry) => Some("Registry Saved".to_owned()),
            _ => None,
        };

        if let Some(token) = token {
            self.clear_progress();
            self.emitted_output = true;
            eprintln!("{}", format_completed_stage_line(&token));
        }
    }

    fn show_spinner(&mut self, message: String) {
        if !self.interactive {
            self.emitted_output = true;
            eprintln!("{}", crate::ui::theme::accent(&message));
            return;
        }

        if self.byte_total.is_some() {
            self.clear_progress();
        }

        let progress_bar = self.progress_bar.get_or_insert_with(|| {
            let progress_bar = new_progress_bar(None);
            progress_bar.set_style(spinner_style());
            progress_bar.enable_steady_tick(Duration::from_millis(100));
            progress_bar
        });
        progress_bar.set_message(crate::ui::theme::accent(&message));
        self.byte_total = None;
    }

    fn show_progress(&mut self, current: u64, total: Option<u64>) {
        self.last_progress_bytes = Some(current);

        if !self.interactive {
            return;
        }

        let total = total.unwrap_or_else(|| current.max(1));
        let replace_progress = self.byte_total != Some(total);

        if replace_progress {
            self.clear_progress();
            let progress_bar = new_progress_bar(Some(total));
            progress_bar.set_style(byte_style());
            self.progress_bar = Some(progress_bar);
            self.byte_total = Some(total);
        }

        if let Some(progress_bar) = &self.progress_bar {
            progress_bar.set_length(total);
            progress_bar.set_position(current.min(total));
        }
    }
}

impl Default for TerminalProgressReporter {
    fn default() -> Self {
        Self::stderr()
    }
}

impl ProgressReporter for TerminalProgressReporter {
    fn report(&mut self, event: &OperationEvent) {
        match event {
            OperationEvent::Started { .. } => {
                if let Some(message) = event_message(event) {
                    self.show_spinner(message);
                }
            }
            OperationEvent::StageChanged { stage, .. } => {
                self.emit_completed_stage_token();
                self.current_stage = Some(*stage);
                if let Some(message) = event_message(event) {
                    self.show_spinner(message);
                }
            }
            OperationEvent::Progress { current, total } => self.show_progress(*current, *total),
            OperationEvent::Warning { .. } | OperationEvent::Failed { .. } => {
                self.clear_progress();
                if let Some(message) = event_message(event) {
                    self.emitted_output = true;
                    let styled = match event {
                        OperationEvent::Warning { .. } => crate::ui::theme::warning_text(&message),
                        OperationEvent::Failed { .. } => crate::ui::theme::error_text(&message),
                        _ => message,
                    };
                    eprintln!("{styled}");
                }
            }
            OperationEvent::Finished { .. } => {
                self.emit_completed_stage_token();
                self.current_stage = None;
                self.last_progress_bytes = None;
                self.clear_progress();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TerminalProgressReporter;
    use crate::ui::progress::{ProgressReporter, format_completed_stage_line};
    use aim_core::app::progress::{OperationEvent, OperationStage};

    #[test]
    fn stage_change_resets_byte_progress_position() {
        let mut reporter = TerminalProgressReporter {
            interactive: true,
            progress_bar: None,
            byte_total: None,
            current_stage: None,
            last_progress_bytes: None,
            emitted_output: false,
        };

        reporter.report(&OperationEvent::Progress {
            current: 98,
            total: Some(100),
        });

        let byte_position = reporter
            .progress_bar
            .as_ref()
            .expect("progress bar created")
            .position();
        assert_eq!(byte_position, 98);

        reporter.report(&OperationEvent::StageChanged {
            stage: OperationStage::StagePayload,
            message: "staging payload".to_owned(),
        });

        let stage_position = reporter
            .progress_bar
            .as_ref()
            .expect("spinner bar retained")
            .position();
        assert_eq!(stage_position, 0);
    }

    #[test]
    fn completed_stage_lines_use_checklist_format() {
        let line = format_completed_stage_line("Payload Staged");

        assert_eq!(
            line,
            format!("{} Payload Staged", crate::ui::theme::success("✓"))
        );
    }
}
