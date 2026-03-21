fn main() {
    let loaded_theme_config = aim_cli::cli::config::AppConfig::load();
    aim_cli::ui::theme::set_active_theme(aim_cli::ui::theme::resolve_theme(
        &loaded_theme_config.config.theme,
    ));
    for warning in loaded_theme_config.warnings {
        eprintln!(
            "{}",
            aim_cli::ui::theme::warning_text(&format!("Config warning: {warning}"))
        );
    }

    let config = match aim_cli::config::load() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    let cli = aim_cli::parse();
    let mut reporter = aim_cli::ui::progress::TerminalProgressReporter::stderr();
    match aim_cli::dispatch_with_reporter_and_config(cli, &config, &mut reporter) {
        Ok(result) => {
            let output = aim_cli::render_with_config(&result, &config);
            if !output.is_empty() {
                if reporter.emitted_output() {
                    println!();
                }
                println!("{output}");
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
