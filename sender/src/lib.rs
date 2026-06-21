mod app;
mod domain;
mod infrastructure;

pub async fn run() -> anyhow::Result<()> {
    app::run().await
}
