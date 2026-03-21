use std::collections::BTreeSet;
use std::io::IsTerminal;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, List, ListItem, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use upm_core::domain::search::{SearchInstallStatus, SearchResult, SearchResults};

use crate::config::{CliConfig, SearchConfig};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserPhase {
    Browsing,
    Confirming,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchRow {
    pub status: SearchInstallStatus,
    pub provider_id: String,
    pub display_name: String,
    pub description: Option<String>,
    pub install_query: String,
    pub version: Option<String>,
    pub selectable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchSelection {
    pub rows: Vec<SearchRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SubmitAction {
    None,
    Confirming,
    Confirmed(SearchSelection),
}

pub struct SearchBrowserState {
    rows: Vec<SearchRow>,
    query_text: String,
    selected: BTreeSet<usize>,
    cursor: usize,
    page_size: usize,
    phase: BrowserPhase,
    numeric_buffer: String,
    status_message: Option<String>,
}

impl SearchBrowserState {
    pub fn new(results: Vec<SearchResult>, config: SearchConfig, page_size: usize) -> Self {
        Self::new_with_query(results, String::new(), config, page_size)
    }

    pub fn new_with_query(
        results: Vec<SearchResult>,
        query_text: String,
        config: SearchConfig,
        page_size: usize,
    ) -> Self {
        let mut rows = results
            .into_iter()
            .map(|result| SearchRow {
                selectable: !matches!(result.install_status, SearchInstallStatus::Installed { .. }),
                status: result.install_status,
                provider_id: result.provider_id,
                display_name: result.display_name,
                description: result.description,
                install_query: result.install_query,
                version: result.version,
            })
            .collect::<Vec<_>>();

        if config.bottom_to_top {
            rows.reverse();
        }

        Self {
            rows,
            query_text,
            selected: BTreeSet::new(),
            cursor: 0,
            page_size: page_size.max(1),
            phase: BrowserPhase::Browsing,
            numeric_buffer: String::new(),
            status_message: None,
        }
    }

    pub fn ordered_rows(&self) -> &[SearchRow] {
        &self.rows
    }

    pub fn query_text(&self) -> &str {
        &self.query_text
    }

    pub fn selected_rows(&self) -> Vec<&SearchRow> {
        self.selected
            .iter()
            .filter_map(|index| self.rows.get(*index))
            .collect()
    }

    pub fn selected_rows_owned(&self) -> Vec<SearchRow> {
        self.selected_rows().into_iter().cloned().collect()
    }

    pub fn selection_expression(&self) -> String {
        compress_selection_ranges(
            &self
                .selected
                .iter()
                .map(|index| index + 1)
                .collect::<Vec<_>>(),
        )
    }

    pub fn selection_prompt_value(&self) -> String {
        if self.numeric_buffer.is_empty() {
            self.selection_expression()
        } else {
            self.numeric_buffer.clone()
        }
    }

    pub fn phase(&self) -> BrowserPhase {
        self.phase
    }

    pub fn cursor_position(&self) -> usize {
        self.cursor
    }

    pub fn selection_count(&self) -> usize {
        self.selected.len()
    }

    pub fn has_selection(&self) -> bool {
        !self.selected.is_empty()
    }

    pub fn numeric_buffer(&self) -> &str {
        &self.numeric_buffer
    }

    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    pub fn page_bounds(&self) -> (usize, usize) {
        let start = (self.cursor / self.page_size) * self.page_size;
        let end = (start + self.page_size).min(self.rows.len());
        (start, end)
    }

    pub fn move_next(&mut self) {
        if self.cursor + 1 < self.rows.len() {
            self.cursor += 1;
        }
    }

    pub fn move_previous(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_to_top(&mut self) {
        self.cursor = 0;
    }

    pub fn move_to_bottom(&mut self) {
        if !self.rows.is_empty() {
            self.cursor = self.rows.len() - 1;
        }
    }

    pub fn page_down(&mut self) {
        if self.rows.is_empty() {
            return;
        }

        let next_page = ((self.cursor / self.page_size) + 1) * self.page_size;
        self.cursor = next_page.min(self.rows.len().saturating_sub(1));
    }

    pub fn page_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(self.cursor % self.page_size);
        self.cursor = self.cursor.saturating_sub(self.page_size);
    }

    pub fn toggle_current_selection(&mut self) {
        if self
            .rows
            .get(self.cursor)
            .is_some_and(|row| !row.selectable)
        {
            self.set_status_message("installed result is not selectable");
            return;
        }

        if !self.selected.insert(self.cursor) {
            self.selected.remove(&self.cursor);
        }

        self.clear_status_message();
    }

    pub fn enter_confirmation(&mut self) -> bool {
        if self.selected.is_empty() {
            return false;
        }

        self.phase = BrowserPhase::Confirming;
        true
    }

    pub fn cancel_confirmation(&mut self) {
        self.phase = BrowserPhase::Browsing;
    }

    pub fn apply_numeric_selection(&mut self, input: &str) -> Result<(), String> {
        let parsed = parse_selection(input, self.rows.len())?;
        self.selected = parsed
            .into_iter()
            .filter(|index| self.rows.get(*index).is_some_and(|row| row.selectable))
            .collect();
        Ok(())
    }

    pub fn submit_selection(&mut self, skip_confirmation: bool) -> SubmitAction {
        if !self.has_selection() {
            self.set_status_message("select at least one result");
            return SubmitAction::None;
        }

        if skip_confirmation {
            return SubmitAction::Confirmed(SearchSelection {
                rows: self.selected_rows_owned(),
            });
        }

        self.enter_confirmation();
        SubmitAction::Confirming
    }

    pub fn push_numeric_input(&mut self, character: char) {
        self.numeric_buffer.push(character);
        self.refresh_selection_from_numeric_buffer();
    }

    pub fn pop_numeric_input(&mut self) {
        self.numeric_buffer.pop();
        self.refresh_selection_from_numeric_buffer();
    }

    pub fn clear_numeric_input(&mut self) {
        self.numeric_buffer.clear();
    }

    pub fn set_status_message(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
    }

    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    fn is_selected(&self, index: usize) -> bool {
        self.selected.contains(&index)
    }

    fn refresh_selection_from_numeric_buffer(&mut self) {
        let trimmed = self.numeric_buffer.trim();
        if trimmed.is_empty() {
            return;
        }

        if let Ok(parsed) = parse_selection(trimmed, self.rows.len()) {
            self.selected = parsed
                .into_iter()
                .filter(|index| self.rows.get(*index).is_some_and(|row| row.selectable))
                .collect();
        }
    }
}

#[derive(Debug)]
pub enum SearchBrowserError {
    Terminal(std::io::Error),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HighlightSegment {
    pub text: String,
    pub is_match: bool,
}

pub fn can_launch(results: &SearchResults) -> bool {
    !results.remote_hits.is_empty()
        && std::io::stdin().is_terminal()
        && std::io::stdout().is_terminal()
}

pub fn run(
    results: &SearchResults,
    config: &CliConfig,
) -> Result<Option<SearchSelection>, SearchBrowserError> {
    let mut stdout = std::io::stdout();
    enable_raw_mode().map_err(SearchBrowserError::Terminal)?;
    execute!(stdout, EnterAlternateScreen).map_err(SearchBrowserError::Terminal)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(SearchBrowserError::Terminal)?;
    let outcome = run_loop(&mut terminal, results, config);

    let leave_screen = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let show_cursor = terminal.show_cursor();
    let disable_raw = disable_raw_mode();

    if let Err(error) = leave_screen {
        return Err(SearchBrowserError::Terminal(error));
    }
    if let Err(error) = show_cursor {
        return Err(SearchBrowserError::Terminal(error));
    }
    if let Err(error) = disable_raw {
        return Err(SearchBrowserError::Terminal(error));
    }

    outcome
}

pub fn format_search_row(
    index: usize,
    row: &SearchRow,
    selected: bool,
    active: bool,
    width: usize,
) -> String {
    let cursor = if active { ">" } else { " " };
    let marker = if selected { "[*]" } else { "[ ]" };
    let status = match &row.status {
        SearchInstallStatus::Available => "",
        SearchInstallStatus::Installed { .. } => "[installed] ",
        SearchInstallStatus::UpdateAvailable { .. } => "[update] ",
    };
    let version = row
        .version
        .as_deref()
        .map(|value| format!("  v{value}"))
        .unwrap_or_default();
    let first_line = format!(
        "{cursor}{marker} {index:>2}. {status}{}{version}",
        row.display_name
    );
    let second_line = match row.description.as_deref() {
        Some(description) => format!("{} - {description}", row.provider_id),
        None => row.provider_id.clone(),
    };
    format!(
        "{}\n{}",
        truncate_line(&first_line, width),
        truncate_line(&format!("       {second_line}"), width)
    )
}

pub fn highlight_segments(text: &str, query: &str) -> Vec<HighlightSegment> {
    let normalized_query = query.trim().to_ascii_lowercase();
    if normalized_query.is_empty() {
        return vec![HighlightSegment {
            text: text.to_owned(),
            is_match: false,
        }];
    }

    let normalized_text = text.to_ascii_lowercase();
    let mut start = 0;
    let mut segments = Vec::new();

    while let Some(relative_match) = normalized_text[start..].find(&normalized_query) {
        let match_start = start + relative_match;
        let match_end = match_start + normalized_query.len();

        if match_start > start {
            segments.push(HighlightSegment {
                text: text[start..match_start].to_owned(),
                is_match: false,
            });
        }

        segments.push(HighlightSegment {
            text: text[match_start..match_end].to_owned(),
            is_match: true,
        });
        start = match_end;
    }

    if start < text.len() {
        segments.push(HighlightSegment {
            text: text[start..].to_owned(),
            is_match: false,
        });
    }

    if segments.is_empty() {
        segments.push(HighlightSegment {
            text: text.to_owned(),
            is_match: false,
        });
    }

    segments
}

pub fn render_confirmation_summary(rows: &[SearchRow]) -> String {
    let mut lines = vec![crate::ui::theme::heading("Confirm Search Selection")];
    lines.push(format!("selected results: {}", rows.len()));
    for row in rows {
        lines.push(format!(
            "{} [{}] {}",
            crate::ui::theme::bullet(&row.display_name),
            row.provider_id,
            row.version
                .as_deref()
                .map(|value| format!("{} (v{value})", row.install_query))
                .unwrap_or_else(|| row.install_query.clone())
        ));
    }
    lines.join("\n")
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    results: &SearchResults,
    config: &CliConfig,
) -> Result<Option<SearchSelection>, SearchBrowserError> {
    let mut state = SearchBrowserState::new_with_query(
        results.remote_hits.clone(),
        results.query_text.clone(),
        config.search.clone(),
        10,
    );

    loop {
        terminal
            .draw(|frame| draw_browser(frame, &state, results, config))
            .map_err(SearchBrowserError::Terminal)?;

        if !event::poll(Duration::from_millis(250)).map_err(SearchBrowserError::Terminal)? {
            continue;
        }

        let Event::Key(key) = event::read().map_err(SearchBrowserError::Terminal)? else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        if let Some(outcome) = handle_key_event(&mut state, key.code, key.modifiers, &config.search)
        {
            return Ok(outcome);
        }
    }
}

fn draw_browser(
    frame: &mut Frame<'_>,
    state: &SearchBrowserState,
    _results: &SearchResults,
    config: &CliConfig,
) {
    let palette = crate::ui::theme::search_browser_palette(&config.theme);

    if state.phase() == BrowserPhase::Confirming {
        let area = centered_rect(frame.area(), 70, 40);
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(render_confirmation_summary(&state.selected_rows_owned()))
                .style(palette.text_style()),
            area,
        );
        return;
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let header = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(1)])
        .split(layout[0]);
    let header_top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(24)])
        .split(header[0]);
    let (start, end) = state.page_bounds();

    frame.render_widget(
        Paragraph::new(Line::styled("Search Results", palette.heading_style())),
        header_top[0],
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                format!(
                    "Showing {}-{} of {}",
                    start + 1,
                    end,
                    state.ordered_rows().len()
                ),
                palette.muted_style(),
            ),
            Line::styled(
                format!("Selected {}", state.selection_count()),
                palette.muted_style(),
            ),
        ])
        .alignment(Alignment::Right),
        header_top[1],
    );
    frame.render_widget(
        Paragraph::new(Line::styled(
            "Enter confirm  Space toggle  j/k move  PgUp/PgDn page  g/G jump  q cancel",
            palette.hint_style(),
        ))
        .wrap(Wrap { trim: true }),
        header[1],
    );

    let width = layout[1].width as usize;
    let items = state.ordered_rows()[start..end]
        .iter()
        .enumerate()
        .map(|(offset, row)| {
            let absolute = start + offset;
            ListItem::new(render_search_row_lines(
                absolute + 1,
                row,
                state.is_selected(absolute),
                state.cursor_position() == absolute,
                width,
                palette,
                state.query_text(),
            ))
        })
        .collect::<Vec<_>>();
    frame.render_widget(List::new(items), layout[1]);

    let status = state.status_message().unwrap_or("");
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Apps to install: ", palette.text_style()),
                Span::styled(state.selection_prompt_value(), palette.text_style()),
                Span::styled("  eg. 1 2 3, 1-3", palette.hint_style()),
            ]),
            Line::styled(status, palette.muted_style()),
        ])
        .wrap(Wrap { trim: true }),
        layout[2],
    );
}

fn render_search_row_lines(
    index: usize,
    row: &SearchRow,
    selected: bool,
    active: bool,
    width: usize,
    palette: crate::ui::theme::SearchBrowserPalette,
    query_text: &str,
) -> Vec<Line<'static>> {
    let cursor = if active { ">" } else { " " };
    let checkbox = if selected { "[*]" } else { "[ ]" };
    let checkbox_style = if selected {
        palette.checkbox_selected_style()
    } else {
        palette.checkbox_idle_style()
    };
    let name_style = if !row.selectable {
        palette.disabled_style()
    } else if active {
        palette.active_name_style()
    } else {
        palette.text_style()
    };
    let index_style = if row.selectable {
        palette.text_style()
    } else {
        palette.disabled_style()
    };

    let mut first_line = vec![
        Span::styled(cursor.to_owned(), palette.cursor_style()),
        Span::raw(" "),
        Span::styled(checkbox.to_owned(), checkbox_style),
        Span::styled(format!(" {index:>2}. "), index_style),
    ];

    match row.status {
        SearchInstallStatus::Available => {}
        SearchInstallStatus::Installed { .. } => {
            first_line.push(Span::styled(
                "[installed] ".to_owned(),
                name_style.add_modifier(Modifier::BOLD),
            ));
        }
        SearchInstallStatus::UpdateAvailable { .. } => {
            first_line.push(Span::styled(
                "[update] ".to_owned(),
                name_style.add_modifier(Modifier::BOLD),
            ));
        }
    }

    push_highlighted_spans(&mut first_line, &row.display_name, query_text, name_style);
    if let Some(version) = &row.version {
        first_line.push(Span::raw("  "));
        first_line.push(Span::styled(format!("v{version}"), palette.version_style()));
    }

    let detail_text = match row.description.as_deref() {
        Some(description) => format!("{} - {description}", row.provider_id),
        None => row.provider_id.clone(),
    };
    let detail_text = truncate_line(&detail_text, width.saturating_sub(7));
    let provider_len = row.provider_id.len().min(detail_text.len());
    let (provider_text, remainder) = detail_text.split_at(provider_len);
    let mut second_line = vec![Span::raw("       ")];
    second_line.push(Span::styled(
        provider_text.to_owned(),
        palette.dim_style().add_modifier(Modifier::BOLD),
    ));
    if !remainder.is_empty() {
        push_highlighted_spans(&mut second_line, remainder, query_text, palette.dim_style());
    }

    vec![Line::from(first_line), Line::from(second_line)]
}

fn handle_key_event(
    state: &mut SearchBrowserState,
    code: KeyCode,
    modifiers: KeyModifiers,
    config: &SearchConfig,
) -> Option<Option<SearchSelection>> {
    if state.phase() == BrowserPhase::Confirming {
        return match code {
            KeyCode::Enter | KeyCode::Char('y') => Some(Some(SearchSelection {
                rows: state.selected_rows_owned(),
            })),
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('n') => {
                state.cancel_confirmation();
                state.set_status_message("confirmation cancelled");
                None
            }
            _ => None,
        };
    }

    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            state.move_previous();
            state.clear_status_message();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.move_next();
            state.clear_status_message();
        }
        KeyCode::PageDown => state.page_down(),
        KeyCode::PageUp => state.page_up(),
        KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => state.page_down(),
        KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => state.page_up(),
        KeyCode::Char('g') => state.move_to_top(),
        KeyCode::Char('G') => state.move_to_bottom(),
        KeyCode::Char(' ') => {
            if state.numeric_buffer().is_empty() {
                state.toggle_current_selection();
            } else if !state.numeric_buffer().ends_with(' ') {
                state.push_numeric_input(' ');
            }
        }
        KeyCode::Char(character)
            if character.is_ascii_digit() || character == ',' || character == '-' =>
        {
            state.push_numeric_input(character);
        }
        KeyCode::Backspace => state.pop_numeric_input(),
        KeyCode::Enter => match state.submit_selection(config.skip_confirmation) {
            SubmitAction::None | SubmitAction::Confirming => {}
            SubmitAction::Confirmed(selection) => return Some(Some(selection)),
        },
        KeyCode::Esc | KeyCode::Char('q') => return Some(None),
        _ => {}
    }

    None
}

fn parse_selection(input: &str, row_count: usize) -> Result<BTreeSet<usize>, String> {
    let mut selected = BTreeSet::new();

    for token in input
        .split(|character: char| character == ',' || character.is_ascii_whitespace())
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        if let Some((start, end)) = token.split_once('-') {
            let start = parse_one_based(start, row_count, input)?;
            let end = parse_one_based(end, row_count, input)?;
            let (from, to) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            for index in from..=to {
                selected.insert(index);
            }
        } else {
            selected.insert(parse_one_based(token, row_count, input)?);
        }
    }

    Ok(selected)
}

fn parse_one_based(token: &str, row_count: usize, original: &str) -> Result<usize, String> {
    let parsed = token
        .parse::<usize>()
        .map_err(|_| format!("invalid selection '{original}'"))?;

    if parsed == 0 || parsed > row_count {
        return Err(format!("invalid selection '{original}'"));
    }

    Ok(parsed - 1)
}

fn push_highlighted_spans(
    target: &mut Vec<Span<'static>>,
    text: &str,
    query: &str,
    base_style: ratatui::style::Style,
) {
    for segment in highlight_segments(text, query) {
        let style = if segment.is_match {
            base_style.add_modifier(Modifier::BOLD)
        } else {
            base_style
        };
        target.push(Span::styled(segment.text, style));
    }
}

fn truncate_line(line: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    let length = line.chars().count();
    if length <= width {
        return line.to_owned();
    }

    if width == 1 {
        return ".".to_owned();
    }

    if width <= 3 {
        return ".".repeat(width);
    }

    let mut truncated = line.chars().take(width - 3).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn compress_selection_ranges(indices: &[usize]) -> String {
    if indices.is_empty() {
        return String::new();
    }

    let mut ranges = Vec::new();
    let mut start = indices[0];
    let mut end = indices[0];

    for &index in &indices[1..] {
        if index == end + 1 {
            end = index;
            continue;
        }

        ranges.push(format_range(start, end));
        start = index;
        end = index;
    }

    ranges.push(format_range(start, end));
    ranges.join(",")
}

fn format_range(start: usize, end: usize) -> String {
    if start == end {
        start.to_string()
    } else {
        format!("{start}-{end}")
    }
}

fn centered_rect(area: Rect, width_percent: u16, height_percent: u16) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height_percent) / 2),
            Constraint::Percentage(height_percent),
            Constraint::Percentage((100 - height_percent) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width_percent) / 2),
            Constraint::Percentage(width_percent),
            Constraint::Percentage((100 - width_percent) / 2),
        ])
        .split(vertical[1])[1]
}
