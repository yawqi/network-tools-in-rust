use anyhow::anyhow;
use ttcp_rs::common::Args;
use ttcp_rs::ttcp_blocking;
fn main() -> anyhow::Result<()> {
    let arg = Args::parse().ok_or(anyhow!("Failed to parse arguments"))?;

    if arg.is_receive() {
        ttcp_blocking::receive(arg)?;
    } else {
        ttcp_blocking::transmit(arg)?;
    }
    Ok(())
}
