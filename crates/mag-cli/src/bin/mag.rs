//! Entry point for `mag` executable.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    mag_cli::run_cli().await
}
