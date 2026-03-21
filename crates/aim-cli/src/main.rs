fn main() {
    let config = match aim_cli::config::load() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    let cli = aim_cli::parse();
    let mut reporter = aim_cli::ui::progress::TerminalProgressReporter::stderr();
    match aim_cli::dispatch_with_reporter(cli, &mut reporter) {
        Ok(result) => {
            let output = aim_cli::render_with_config(&result, &config);
            if !output.is_empty() {
                println!("{output}");
            }
        }
        Err(error) => {
            eprintln!("{error:?}");
            std::process::exit(1);
        }
    }
}
