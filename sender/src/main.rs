use clap::Parser;

mod campaign;
mod ledger;
mod mailbox;
mod message;
mod prospect;
mod template;
mod throttle;

use campaign::Campaign;
use ledger::Ledger;
use mailbox::Mailbox;
use prospect::Prospects;
use template::Template;
use throttle::Throttle;

const DEFAULT_ENV: &str = concat!(
    "MAIL_SUBJECT=\"Une idée de vitrine web pour votre cabinet — Maître {{name}}\"\n",
    "MAIL_BODY_PATH=\"storage/body.md\"\n",
);

const DEFAULT_BODY: &str = "Bonjour Maître {{name}},

Je me permets de vous contacter car j’ai préparé un aperçu simple de vitrine web pour un cabinet comme le vôtre, avec une présentation professionnelle, vos domaines d’intervention et un accès rapide aux informations de contact.

Vous pouvez le consulter ici :

{{link}}

C’est une maquette libre d’accès, sans engagement. L’objectif est simplement de vous montrer comment votre cabinet pourrait être présenté en ligne de façon claire et rassurante.

Si l’idée vous intéresse, je serais ravi d’en discuter avec vous.

Cordialement,
[coordonnées]
";

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long)]
    init: bool,
    #[arg(long, default_value = "storage/prospects.json")]
    prospects: String,
    #[arg(long, default_value = "storage/sent.json")]
    ledger: String,
    #[arg(long, default_value = ".env")]
    env_file: String,
    #[arg(long)]
    base_url: Option<String>,
    #[arg(long)]
    from: Option<String>,
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

    if cli.init {
        initialize()?;
        return Ok(());
    }

    require_files(&[&cli.env_file, &cli.prospects, &cli.ledger])?;
    dotenvy::from_path(&cli.env_file)?;

    let body_path = std::env::var("MAIL_BODY_PATH")
        .map_err(|_| anyhow::anyhow!("MAIL_BODY_PATH is missing from {}", cli.env_file))?;
    require_files(&[&body_path])?;

    if !cli.dry_run {
        require_files(&["storage/credentials.json"])?;
    }

    let base_url = cli
        .base_url
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--base-url is required unless --init is used"))?;
    let from = cli
        .from
        .ok_or_else(|| anyhow::anyhow!("--from is required unless --init is used"))?;

    let prospects = Prospects::load(&cli.prospects)?;
    let mut ledger = Ledger::open(&cli.ledger)?;
    let template = Template::load()?;
    let throttle = Throttle::new(cli.min_delay, cli.max_delay)?;

    let mailbox = if cli.dry_run {
        None
    } else {
        Some(Mailbox::authenticate(from).await?)
    };

    let mut campaign = match mailbox.as_ref() {
        Some(mailbox) => Campaign::delivery(
            &prospects,
            &mut ledger,
            &template,
            &throttle,
            mailbox,
            base_url,
            cli.daily_cap,
        ),
        None => Campaign::preview(
            &prospects,
            &mut ledger,
            &template,
            &throttle,
            base_url,
            cli.daily_cap,
        ),
    };

    campaign.run().await
}

fn initialize() -> anyhow::Result<()> {
    std::fs::create_dir_all("storage")?;

    create_if_missing(".env", DEFAULT_ENV)?;
    create_if_missing("storage/body.md", DEFAULT_BODY)?;
    create_if_missing("storage/prospects.json", "[]\n")?;
    create_if_missing("storage/sent.json", "{}\n")?;

    println!(
        "Setup complete. Add prospects to storage/prospects.json and download Google OAuth credentials to storage/credentials.json."
    );
    Ok(())
}

fn create_if_missing(path: &str, contents: &str) -> anyhow::Result<()> {
    use std::io::Write;

    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(mut file) => {
            file.write_all(contents.as_bytes())?;
            println!("created {path}");
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            println!("kept existing {path}");
        }
        Err(error) => return Err(error.into()),
    }

    Ok(())
}

fn require_files(paths: &[&str]) -> anyhow::Result<()> {
    let missing: Vec<_> = paths
        .iter()
        .filter(|path| !std::path::Path::new(path).is_file())
        .copied()
        .collect();

    anyhow::ensure!(
        missing.is_empty(),
        "missing required file(s): {}\nRun `prospect-mailer --init` to generate local runtime files.",
        missing.join(", ")
    );
    Ok(())
}
