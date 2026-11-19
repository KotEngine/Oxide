use oxide::server::OxideServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("oxide=debug"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    let server = OxideServer::new("0.0.0.0:19132".parse()?).await?;
    server.start().await
}
