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
    ProgressStyle::with_template("{bar:32.cyan/blue} {bytes}/{total_bytes} {msg}")
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
}

impl TerminalProgressReporter {
    pub fn stderr() -> Self {
        Self {
            interactive: std::io::stderr().is_terminal(),
            progress_bar: None,
            byte_total: None,
        }
    }

    fn clear_progress(&mut self) {
        if let Some(progress_bar) = self.progress_bar.take() {
            progress_bar.finish_and_clear();
        }
        self.byte_total = None;
    }

    fn show_spinner(&mut self, message: String) {
        if !self.interactive {
            eprintln!("{message}");
            return;
        }

        let progress_bar = self.progress_bar.get_or_insert_with(|| {
            let progress_bar = new_progress_bar(None);
            progress_bar.set_style(spinner_style());
            progress_bar.enable_steady_tick(Duration::from_millis(100));
            progress_bar
        });
        progress_bar.set_message(message);
        self.byte_total = None;
    }

    fn show_progress(&mut self, current: u64, total: Option<u64>) {
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
            OperationEvent::Started { .. } | OperationEvent::StageChanged { .. } => {
                if let Some(message) = event_message(event) {
                    self.show_spinner(message);
                }
            }
            OperationEvent::Progress { current, total } => self.show_progress(*current, *total),
            OperationEvent::Warning { .. } | OperationEvent::Failed { .. } => {
                self.clear_progress();
                if let Some(message) = event_message(event) {
                    eprintln!("{message}");
                }
            }
            OperationEvent::Finished { .. } => self.clear_progress(),
        }
    }
}
