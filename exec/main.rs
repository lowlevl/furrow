use clap::Parser;
use color_eyre::eyre;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod hooks;
mod server;

#[derive(Debug, Parser)]
#[command(multicall = true, rename_all = "kebab-case")]
pub enum Cli {
    #[command(name = env!("CARGO_PKG_NAME"))]
    Server(server::Server),

    #[command(flatten)]
    Hooks(hooks::Hooks),
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    match Cli::parse() {
        Cli::Server(server) => {
            // Set-up the pretty-printed error handler
            color_eyre::install()?;

            // Set-up the log and traces handler
            tracing_subscriber::registry()
                .with(fmt::layer())
                .with(EnvFilter::from_default_env())
                .init();

            // Finally configure and start the server
            server.start().await
        }
        Cli::Hooks(hook) => hook.run().await,
    }
}
