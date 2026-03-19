fn main() {
    let cli = aim_cli::parse();
    match aim_cli::dispatch(cli) {
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
