use oxide::server::OxideServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("oxide=info".parse()?))
        .init();

    let mut server = OxideServer::new("0.0.0.0:19132".parse()?).await?;
    server.start().await
}
