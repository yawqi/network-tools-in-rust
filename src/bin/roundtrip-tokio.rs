use tracing::subscriber;
use ttcp_rs::roundtrip::*;

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = RoundTripArgs::parse();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .pretty()
        .finish();

    subscriber::set_global_default(subscriber)?;

    if args.is_server() {
        async_server(args.host(), args.port()).await?;
    } else {
        async_client(args.host(), args.port()).await?;
    }

    Ok(())
}