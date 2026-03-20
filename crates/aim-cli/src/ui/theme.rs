use std::sync::OnceLock;

use console::{Style, true_colors_enabled};
use dialoguer::theme::ColorfulTheme;

use crate::cli::config::ThemeConfig;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ColorValue {
    Named(String),
    Rgb(u8, u8, u8),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StyleSpec {
    pub bold: bool,
    pub dim: bool,
    pub foreground: Option<ColorValue>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Theme {
    pub heading: StyleSpec,
    pub accent: StyleSpec,
    pub muted: StyleSpec,
    pub label: StyleSpec,
    pub bullet: StyleSpec,
    pub success: StyleSpec,
    pub warning: StyleSpec,
    pub error: StyleSpec,
    pub progress_spinner: StyleSpec,
    pub progress_bar: StyleSpec,
    pub progress_bar_unfilled: StyleSpec,
}

static ACTIVE_THEME: OnceLock<Theme> = OnceLock::new();

impl Default for Theme {
    fn default() -> Self {
        Self {
            heading: parse_style_spec("bold #7c3aed").expect("valid default heading style"),
            accent: parse_style_spec("#8b5cf6").expect("valid default accent style"),
            muted: parse_style_spec("dim #75658a").expect("valid default muted style"),
            label: parse_style_spec("bold #c4b5fd").expect("valid default label style"),
            bullet: StyleSpec::default(),
            success: parse_style_spec("green").expect("valid default success style"),
            warning: parse_style_spec("yellow").expect("valid default warning style"),
            error: parse_style_spec("red").expect("valid default error style"),
            progress_spinner: parse_style_spec("#8b5cf6").expect("valid default spinner style"),
            progress_bar: parse_style_spec("#8b5cf6").expect("valid default bar style"),
            progress_bar_unfilled: parse_style_spec("#75658a")
                .expect("valid default unfilled bar style"),
        }
    }
}

pub fn resolve_theme(config: &ThemeConfig) -> Theme {
    let mut theme = Theme::default();
    override_spec(&mut theme.heading, config.heading.as_deref());
    override_spec(&mut theme.accent, config.accent.as_deref());
    override_spec(&mut theme.muted, config.muted.as_deref());
    override_spec(&mut theme.label, config.label.as_deref());
    override_spec(&mut theme.bullet, config.bullet.as_deref());
    override_spec(&mut theme.success, config.success.as_deref());
    override_spec(&mut theme.warning, config.warning.as_deref());
    override_spec(&mut theme.error, config.error.as_deref());
    override_spec(
        &mut theme.progress_spinner,
        config.progress_spinner.as_deref(),
    );
    override_spec(&mut theme.progress_bar, config.progress_bar.as_deref());
    override_spec(
        &mut theme.progress_bar_unfilled,
        config.progress_bar_unfilled.as_deref(),
    );
    theme
}

pub fn set_active_theme(theme: Theme) {
    let _ = ACTIVE_THEME.set(theme);
}

pub fn current_theme() -> Theme {
    ACTIVE_THEME.get().cloned().unwrap_or_default()
}

pub fn dialog_theme() -> ColorfulTheme {
    ColorfulTheme::default()
}

pub fn heading(title: &str) -> String {
    apply_style_spec(title, &current_theme().heading)
}

pub fn label(title: &str) -> String {
    apply_style_spec(&format!("{title}:"), &current_theme().label)
}

pub fn muted(message: &str) -> String {
    apply_style_spec(message, &current_theme().muted)
}

pub fn bullet(message: &str) -> String {
    format!("- {message}")
}

pub fn accent(message: &str) -> String {
    apply_style_spec(message, &current_theme().accent)
}

pub fn success(message: &str) -> String {
    apply_style_spec(message, &current_theme().success)
}

pub fn warning_text(message: &str) -> String {
    apply_style_spec(message, &current_theme().warning)
}

pub fn error_text(message: &str) -> String {
    apply_style_spec(message, &current_theme().error)
}

pub fn indicatif_color_key(spec: &StyleSpec) -> &'static str {
    match spec.foreground.as_ref() {
        Some(ColorValue::Named(name)) => match name.as_str() {
            "black" | "stone" => "black",
            "red" => "red",
            "green" => "green",
            "yellow" | "amber" | "sand" => "yellow",
            "blue" => "blue",
            "magenta" => "magenta",
            "cyan" | "teal" => "cyan",
            "white" => "white",
            _ => "white",
        },
        Some(ColorValue::Rgb(red, green, blue)) => nearest_indicatif_color(*red, *green, *blue),
        None => "white",
    }
}

pub fn parse_style_spec(input: &str) -> Result<StyleSpec, String> {
    let mut spec = StyleSpec::default();

    for token in input.split_whitespace() {
        match token {
            "bold" => spec.bold = true,
            "dim" => spec.dim = true,
            color => spec.foreground = Some(parse_color_value(color)?),
        }
    }

    Ok(spec)
}

pub fn apply_style_spec(message: &str, spec: &StyleSpec) -> String {
    let mut style = Style::new();
    if spec.bold {
        style = style.bold();
    }
    if spec.dim {
        style = style.dim();
    }
    if let Some(color) = &spec.foreground {
        style = apply_color(style, color);
    }
    style.apply_to(message).to_string()
}

fn override_spec(target: &mut StyleSpec, value: Option<&str>) {
    if let Some(value) = value
        && let Ok(spec) = parse_style_spec(value)
    {
        *target = spec;
    }
}

fn parse_color_value(token: &str) -> Result<ColorValue, String> {
    if let Some(hex) = token.strip_prefix('#') {
        if hex.len() != 6 {
            return Err(format!("invalid hex color: {token}"));
        }

        let red = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| format!("invalid hex color: {token}"))?;
        let green = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| format!("invalid hex color: {token}"))?;
        let blue = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| format!("invalid hex color: {token}"))?;
        return Ok(ColorValue::Rgb(red, green, blue));
    }

    if is_named_color(token) {
        return Ok(ColorValue::Named(token.to_owned()));
    }

    Err(format!("unknown color token: {token}"))
}

fn is_named_color(token: &str) -> bool {
    matches!(
        token,
        "black"
            | "red"
            | "green"
            | "yellow"
            | "blue"
            | "magenta"
            | "cyan"
            | "white"
            | "amber"
            | "teal"
            | "sand"
            | "stone"
    )
}

fn apply_color(style: Style, color: &ColorValue) -> Style {
    match color {
        ColorValue::Named(name) => apply_named_color(style, name),
        ColorValue::Rgb(red, green, blue) => {
            if true_colors_enabled() {
                style.true_color(*red, *green, *blue)
            } else {
                style.color256(rgb_to_ansi256(*red, *green, *blue))
            }
        }
    }
}

fn apply_named_color(style: Style, name: &str) -> Style {
    match name {
        "black" => style.black(),
        "red" => style.red(),
        "green" => style.green(),
        "yellow" => style.yellow(),
        "blue" => style.blue(),
        "magenta" => style.magenta(),
        "cyan" => style.cyan(),
        "white" => style.white(),
        "amber" => apply_color(style, &ColorValue::Rgb(210, 139, 38)),
        "teal" => apply_color(style, &ColorValue::Rgb(47, 142, 138)),
        "sand" => apply_color(style, &ColorValue::Rgb(231, 197, 138)),
        "stone" => apply_color(style, &ColorValue::Rgb(111, 98, 83)),
        _ => style,
    }
}

fn rgb_to_ansi256(red: u8, green: u8, blue: u8) -> u8 {
    let red = ((red as f32 / 255.0) * 5.0).round() as u8;
    let green = ((green as f32 / 255.0) * 5.0).round() as u8;
    let blue = ((blue as f32 / 255.0) * 5.0).round() as u8;
    16 + (36 * red) + (6 * green) + blue
}

fn nearest_indicatif_color(red: u8, green: u8, blue: u8) -> &'static str {
    const COLORS: [(&str, (u8, u8, u8)); 8] = [
        ("black", (0, 0, 0)),
        ("red", (205, 49, 49)),
        ("green", (13, 188, 121)),
        ("yellow", (229, 229, 16)),
        ("blue", (36, 114, 200)),
        ("magenta", (188, 63, 188)),
        ("cyan", (17, 168, 205)),
        ("white", (229, 229, 229)),
    ];

    COLORS
        .iter()
        .min_by_key(|(_, (target_red, target_green, target_blue))| {
            let red_distance = red as i32 - *target_red as i32;
            let green_distance = green as i32 - *target_green as i32;
            let blue_distance = blue as i32 - *target_blue as i32;
            red_distance * red_distance
                + green_distance * green_distance
                + blue_distance * blue_distance
        })
        .map(|(name, _)| *name)
        .unwrap_or("white")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_named_theme_value() {
        let spec = parse_style_spec("amber").unwrap();
        assert_eq!(spec.foreground, Some(ColorValue::Named("amber".to_owned())));
        assert!(!spec.bold);
    }

    #[test]
    fn parses_hex_theme_value() {
        let spec = parse_style_spec("#d28b26").unwrap();
        assert_eq!(spec.foreground, Some(ColorValue::Rgb(210, 139, 38)));
    }

    #[test]
    fn parses_bold_hex_theme_value() {
        let spec = parse_style_spec("bold #d28b26").unwrap();
        assert!(spec.bold);
        assert_eq!(spec.foreground, Some(ColorValue::Rgb(210, 139, 38)));
    }

    #[test]
    fn invalid_override_falls_back_to_default_theme() {
        let theme = resolve_theme(&ThemeConfig {
            heading: Some("bogus".to_owned()),
            ..ThemeConfig::default()
        });

        assert_eq!(theme.heading, Theme::default().heading);
    }

    #[test]
    fn default_theme_uses_purple_led_palette() {
        let theme = Theme::default();

        assert_eq!(theme.heading, parse_style_spec("bold #7c3aed").unwrap());
        assert_eq!(theme.accent, parse_style_spec("#8b5cf6").unwrap());
        assert_eq!(theme.label, parse_style_spec("bold #c4b5fd").unwrap());
        assert_eq!(theme.progress_spinner, parse_style_spec("#8b5cf6").unwrap());
        assert_eq!(theme.progress_bar, parse_style_spec("#8b5cf6").unwrap());
        assert_eq!(
            theme.progress_bar_unfilled,
            parse_style_spec("#75658a").unwrap()
        );
    }
}
