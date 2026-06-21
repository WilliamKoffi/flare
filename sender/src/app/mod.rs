mod cli;
mod workspace;

use clap::Parser;

use crate::domain::campaign::{Campaign, TestMail};
use crate::domain::prospect::{self, Prospects};
use crate::domain::template::Template;
use crate::domain::warmup::Warmup;
use crate::infrastructure::client::Client;
use crate::infrastructure::ledger::Ledger;
use crate::infrastructure::link::Links;
use cli::{
    CampaignAction, Cli, Delivery, Entity, LinkAction, Prepare, ProspectAction, WorkspaceAction,
};

pub(crate) async fn run() -> anyhow::Result<()> {
    execute(Cli::parse()).await
}

async fn execute(cli: Cli) -> anyhow::Result<()> {
    match cli.entity {
        Entity::Workspace { action } => match action {
            WorkspaceAction::Setup => workspace::setup(),
        },
        Entity::Prospect { action } => match action {
            ProspectAction::Export(args) => {
                prospect::export_emails(&args.source, &args.output)?;
                println!("exported emails to {}", args.output);
                Ok(())
            }
        },
        Entity::Link { action } => match action {
            LinkAction::Prepare(args) => prepare(args).await,
        },
        Entity::Campaign { action } => match action {
            CampaignAction::Send(args) => deliver(args, false).await,
            CampaignAction::Simulate(args) => deliver(args, true).await,
        },
    }
}

async fn prepare(args: Prepare) -> anyhow::Result<()> {
    workspace::require(&[&args.environment, &args.source])?;
    dotenvy::from_path(&args.environment)?;

    let address = configured_value(args.address, "BASE_URL", &args.environment, "--address")?;
    let prospects = Prospects::load(&args.source)?;
    let links = Links::load(address, args.shorten)?;
    let mut generated = Vec::new();

    for prospect in prospects.iter() {
        let url = links.for_prospect(prospect).await?;
        println!("prepared {} <{}>", prospect.name, prospect.email);
        generated.push((prospect.id.clone(), url));
    }

    prospect::store_urls(&args.source, &generated)?;
    println!("updated {} URL(s) in {}", generated.len(), args.source);
    Ok(())
}

async fn deliver(args: Delivery, simulate: bool) -> anyhow::Result<()> {
    workspace::require(&[&args.environment, &args.source, &args.ledger])?;
    dotenvy::from_path(&args.environment)?;

    let body_path = std::env::var("MAIL_BODY_PATH")
        .map_err(|_| anyhow::anyhow!("MAIL_BODY_PATH is missing from {}", args.environment))?;
    workspace::require(&[&body_path])?;

    if !simulate {
        workspace::require(&["storage/credentials.json"])?;
    }

    let sender = configured_value(args.sender, "SENDER_EMAIL", &args.environment, "--sender")?;

    let prospects = Prospects::load(&args.source)?;
    prospects.require_urls()?;
    let ledger = Ledger::open(&args.ledger)?;
    let template = Template::load()?;
    let test = TestMail::load()?;
    let warmup = Warmup::load(&args.warmup)?;

    let client = if simulate {
        None
    } else {
        Some(Client::authenticate(sender).await?)
    };

    Campaign::new(
        prospects,
        ledger,
        template,
        client,
        args.capacity,
        args.minimum,
        args.maximum,
        test,
        warmup,
    )
    .launch()
    .await
}

fn configured_value(
    argument: Option<String>,
    variable: &str,
    environment: &str,
    flag: &str,
) -> anyhow::Result<String> {
    argument
        .or_else(|| std::env::var(variable).ok())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("{variable} missing from {environment} (or pass {flag})"))
}
