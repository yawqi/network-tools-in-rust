use anyhow::Error;
use clap::Parser;

use ttcp_rs::common::setup_tracing;
use ttcp_rs::netcat::NetcatArgs;

#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_tracing();
    let args = NetcatArgs::parse();
    ttcp_rs::netcat::run(args).await?;
    Ok(())
}