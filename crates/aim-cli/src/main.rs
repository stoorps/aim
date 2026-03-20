fn main() {
    let cli = aim_cli::parse();
    let mut reporter = aim_cli::ui::progress::TerminalProgressReporter::stderr();
    match aim_cli::dispatch_with_reporter(cli, &mut reporter) {
        Ok(result) => {
            let output = aim_cli::render(&result);
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
