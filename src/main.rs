use clap::Parser;

mod ledger;
mod mailbox;
mod prospect;
mod throttle;

use ledger::Ledger;
use mailbox::Mailbox;
use prospect::Prospect;
use throttle::Throttle;

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long, default_value = "prospects.json")]
    prospects: String,
    #[arg(long, default_value = "sent.json")]
    ledger: String,
    #[arg(long, default_value = "template.md")]
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

    let prospects = Prospect::load_all(&cli.prospects)?;
    let mut ledger = Ledger::open(&cli.ledger)?;
    let template = std::fs::read_to_string(&cli.template)?;
    let throttle = Throttle::new(cli.min_delay, cli.max_delay);

    let mailbox = if cli.dry_run {
        None
    } else {
        Some(Mailbox::authenticate(cli.from.clone()).await?)
    };

    let mut sent_today = 0usize;

    for prospect in &prospects {
        if ledger.has_sent(&prospect.id) {
            println!("⏭  {} — déjà envoyé", prospect.nom);
            continue;
        }

        if sent_today >= cli.daily_cap {
            println!("🛑 Quota journalier atteint ({}).", cli.daily_cap);
            break;
        }

        let link = prospect.link(&cli.base_url);
        let body = render_template(&template, prospect, &link);
        let subject = format!("Présentation web — Maître {}", prospect.nom);

        if cli.dry_run {
            println!("\n--- DRY RUN : {} <{}> ---", prospect.nom, prospect.email);
            println!("Sujet : {}", subject);
            println!("{}", body);
            println!("--- fin ---\n");
        } else {
            let mailbox = mailbox
                .as_ref()
                .expect("mailbox is present when not dry_run");
            mailbox.send(&prospect.email, &subject, &body).await?;
            ledger.record(&prospect.id)?;
            println!("✓ Envoyé à {} <{}>", prospect.nom, prospect.email);
            sent_today += 1;
            throttle.wait().await;
        }
    }

    Ok(())
}

fn render_template(template: &str, prospect: &Prospect, link: &str) -> String {
    template
        .replace("{{nom}}", &prospect.nom)
        .replace("{{specialite}}", &prospect.specialite)
        .replace("{{link}}", link)
}
