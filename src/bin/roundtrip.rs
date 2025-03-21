use ttcp_rs::roundtrip::{self, RoundTripArgs};

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = RoundTripArgs::parse();
    if args.is_server() {
        roundtrip::server(args.host(), args.port())?;
    } else {
        roundtrip::client(args.host(), args.port())?;
    }
    Ok(())
}