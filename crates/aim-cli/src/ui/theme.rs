use console::style;
use dialoguer::theme::ColorfulTheme;

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
