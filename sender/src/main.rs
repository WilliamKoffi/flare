#[tokio::main]
async fn main() -> anyhow::Result<()> {
    prospect_mailer::run().await
}
