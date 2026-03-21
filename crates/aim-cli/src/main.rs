fn main() {
    let loaded_config = aim_cli::cli::config::AppConfig::load();
    aim_cli::ui::theme::set_active_theme(aim_cli::ui::theme::resolve_theme(
        &loaded_config.config.theme,
    ));
    for warning in loaded_config.warnings {
        eprintln!(
            "{}",
            aim_cli::ui::theme::warning_text(&format!("Config warning: {warning}"))
        );
    }

    let cli = aim_cli::parse();
    let mut reporter = aim_cli::ui::progress::TerminalProgressReporter::stderr();
    match aim_cli::dispatch_with_reporter(cli, &mut reporter) {
        Ok(result) => {
            let output = aim_cli::render(&result);
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
