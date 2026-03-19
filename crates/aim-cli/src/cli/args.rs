use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "aim")]
#[command(about = "AppImage Manager")]
pub struct Cli {
    #[arg(global = true, long = "system", conflicts_with = "user")]
    pub system: bool,

    #[arg(global = true, long = "user", conflicts_with = "system")]
    pub user: bool,

    #[command(subcommand)]
    pub command: Option<Command>,

    pub query: Option<String>,
}

impl Cli {
    pub fn is_review_update_flow(&self) -> bool {
        matches!(self.command, Some(Command::Update))
            || (self.command.is_none() && self.query.is_none())
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Remove { query: String },
    List,
    Update,
}
