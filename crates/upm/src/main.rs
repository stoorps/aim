fn main() {
    let loaded_theme_config = upm::cli::config::AppConfig::load();
    upm::ui::theme::set_active_theme(upm::ui::theme::resolve_theme(
        &loaded_theme_config.config.theme,
    ));
    for warning in loaded_theme_config.warnings {
        eprintln!(
            "{}",
            upm::ui::theme::warning_text(&format!("Config warning: {warning}"))
        );
    }

    let config = match upm::config::load() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    let cli = upm::parse();
    let mut reporter = upm::ui::progress::TerminalProgressReporter::stderr();
    match upm::dispatch_with_reporter_and_config(cli, &config, &mut reporter) {
        Ok(result) => {
            let output = upm::render_with_config(&result, &config);
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
