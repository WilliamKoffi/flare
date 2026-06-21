use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "prospect-mailer")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) entity: Entity,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Entity {
    /// Workspace lifecycle (e.g., .env, storage/, templates)
    Workspace {
        #[command(subcommand)]
        action: WorkspaceAction,
    },
    /// Prospect data management
    Prospect {
        #[command(subcommand)]
        action: ProspectAction,
    },
    /// Encryption and link generation
    Link {
        #[command(subcommand)]
        action: LinkAction,
    },
    /// Campaign delivery and simulation
    Campaign {
        #[command(subcommand)]
        action: CampaignAction,
    },
}

#[derive(Subcommand, Debug)]
pub(crate) enum WorkspaceAction {
    Setup,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ProspectAction {
    Export(Export),
}

#[derive(Args, Debug)]
pub(crate) struct Export {
    #[arg(long, default_value = "storage/prospects.toml")]
    pub(crate) source: String,
    #[arg(long, default_value = "storage/prospects-emails.csv")]
    pub(crate) output: String,
}

#[derive(Subcommand, Debug)]
pub(crate) enum LinkAction {
    Prepare(Prepare),
}

#[derive(Args, Debug)]
pub(crate) struct Prepare {
    #[arg(long, default_value = "storage/prospects.toml")]
    pub(crate) source: String,
    #[arg(long, default_value = ".env")]
    pub(crate) environment: String,
    #[arg(long)]
    pub(crate) address: Option<String>,
    #[arg(long)]
    pub(crate) shorten: bool,
}

#[derive(Subcommand, Debug)]
pub(crate) enum CampaignAction {
    Send(Delivery),
    Simulate(Delivery),
}

#[derive(Args, Debug)]
pub(crate) struct Delivery {
    #[arg(long, default_value = "storage/prospects.toml")]
    pub(crate) source: String,
    #[arg(long, default_value = "storage/sent.json")]
    pub(crate) ledger: String,
    #[arg(long, default_value = ".env")]
    pub(crate) environment: String,
    #[arg(long)]
    pub(crate) sender: Option<String>,
    #[arg(long, default_value_t = 600)]
    pub(crate) minimum: u64,
    #[arg(long, default_value_t = 1800)]
    pub(crate) maximum: u64,
    #[arg(long, default_value_t = 15)]
    pub(crate) capacity: usize,
    #[arg(long, default_value = "storage/warmup.toml")]
    pub(crate) warmup: String,
}
