#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().json().with_target(false).init();
    let host = together_session_server::Host::new(std::env::var("TOGETHER_PC_TOKEN")?, std::env::var("TOGETHER_PHONE_TOKEN")?)?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:47820").await?;
    tracing::info!(event="server_started", address="127.0.0.1:47820", protocol=1);
    host.serve(listener).await
}
