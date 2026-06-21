use clap::Parser;

mod campaign;
mod ledger;
mod mailbox;
mod prospect;
mod template;
mod throttle;

use campaign::Campaign;
use ledger::Ledger;
use mailbox::Mailbox;
use prospect::Prospects;
use template::Template;
use throttle::Throttle;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long, default_value = "storage/prospects.json")]
    prospects: String,
    #[arg(long, default_value = "storage/sent.json")]
    ledger: String,
    #[arg(long, default_value = "storage/template.md")]
    template: String,
    #[arg(long)]
    base_url: String,
    #[arg(long)]
    from: String,
    #[arg(long, default_value_t = 600)]
    min_delay: u64,
    #[arg(long, default_value_t = 1800)]
    max_delay: u64,
    #[arg(long, default_value_t = 15)]
    daily_cap: usize,
    #[arg(long)]
    dry_run: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.min_delay > cli.max_delay {
        anyhow::bail!("min_delay must be <= max_delay");
    }

    let prospects = Prospects::load(&cli.prospects)?;
    let mut ledger = Ledger::open(&cli.ledger)?;
    let template = Template::load(&cli.template)?;
    let throttle = Throttle::new(cli.min_delay, cli.max_delay);

    let mailbox = if cli.dry_run {
        None
    } else {
        Some(Mailbox::authenticate(cli.from.clone()).await?)
    };

    Campaign {
        prospects: &prospects,
        ledger: &mut ledger,
        template: &template,
        throttle: &throttle,
        mailbox: mailbox.as_ref(),
        base: &cli.base_url,
        cap: cli.daily_cap,
        preview: cli.dry_run,
    }
    .run()
    .await
}
