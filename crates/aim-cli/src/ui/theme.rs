use console::style;
use dialoguer::theme::ColorfulTheme;
use ratatui::style::{Color, Modifier, Style};

use crate::config::ThemeConfig;

pub fn dialog_theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

pub fn heading(title: &str) -> String {
    style(title).bold().to_string()
}

pub fn label(title: &str) -> String {
    style(format!("{title}:")).bold().to_string()
}

pub fn muted(message: &str) -> String {
    style(message).dim().to_string()
}

pub fn bullet(message: &str) -> String {
    format!("- {message}")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SearchBrowserPalette {
    accent: Color,
    accent_secondary: Color,
    dim: Color,
}

pub fn search_browser_palette(config: &ThemeConfig) -> SearchBrowserPalette {
    SearchBrowserPalette {
        accent: parse_color(&config.accent).unwrap_or(Color::Rgb(179, 136, 255)),
        accent_secondary: parse_color(&config.accent_secondary)
            .unwrap_or(Color::Rgb(213, 194, 255)),
        dim: parse_color(&config.dim).unwrap_or(Color::Rgb(127, 115, 150)),
    }
}

impl SearchBrowserPalette {
    pub fn heading_style(self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn hint_style(self) -> Style {
        Style::default().fg(self.dim)
    }

    pub fn muted_style(self) -> Style {
        Style::default().fg(self.dim)
    }

    pub fn text_style(self) -> Style {
        Style::default().fg(Color::White)
    }

    pub fn dim_style(self) -> Style {
        Style::default().fg(self.dim)
    }

    pub fn checkbox_selected_style(self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn checkbox_idle_style(self) -> Style {
        Style::default().fg(self.dim)
    }

    pub fn version_style(self) -> Style {
        Style::default().fg(self.accent_secondary)
    }

    pub fn tag_style(self) -> Style {
        Style::default().add_modifier(Modifier::BOLD)
    }

    pub fn cursor_style(self) -> Style {
        Style::default()
            .fg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn active_name_style(self) -> Style {
        self.text_style().add_modifier(Modifier::BOLD)
    }

    pub fn disabled_style(self) -> Style {
        self.dim_style()
    }
}

fn parse_color(value: &str) -> Option<Color> {
    let hex = value.trim().strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }

    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(red, green, blue))
}
