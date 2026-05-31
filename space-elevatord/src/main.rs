#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("space_elevatord=info".parse().unwrap()),
        )
        .init();
    space_elevatord::server::run().await
}
