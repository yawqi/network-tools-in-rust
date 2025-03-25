use clap::Parser;

pub fn setup_tracing() {
    use tracing_subscriber::fmt::Subscriber;
    use tracing_subscriber::EnvFilter;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
pub struct Args {
    /// The port to connect to
    #[arg(short, long, value_name = "port number", default_value = "12345")]
    port: Option<u16>,
    /// If we are client, and the address of the server
    #[arg(short, long, value_name = "server address", default_missing_value = "127.0.0.1")]
    transmit: Option<String>,
    /// If we are server
    #[arg(short, long)]
    recv: bool,
    /// Packet size
    #[arg(short, long, value_name = "packet size", default_value = "10240000")]
    length: Option<u32>,
    /// Number of packets to send
    #[arg(short, long, value_name = "packet count", default_value = "10000")]
    count: Option<u32>,
}

impl Args {
    pub fn parse() -> Option<Self> {
        let args = <Args as Parser>::parse();
        if args.recv {
            if args.transmit.is_some() {
                eprintln!("Cannot specify --transmit when --recv is set");
                return None;
            }
        } else {
            if args.transmit.is_none() {
                eprintln!("Must specify --transmit when --recv is not set");
                return None;
            }
        }

        Some(args)
    }

    pub fn is_receive(&self) -> bool {
        self.recv
    }

    pub fn is_transmit(&self) -> bool {
        !self.recv
    }

    pub fn get_server_address(&self) -> Option<&str> {
        self.transmit.as_ref().map(String::as_str).or(Some("127.0.0.1"))
    }

    pub fn get_port(&self) -> u16 {
        self.port.unwrap()
    }

    pub fn get_packet_size(&self) -> u32 {
        self.length.unwrap()
    }

    pub fn get_packet_count(&self) -> u32 {
        self.count.unwrap()
    }
}

#[repr(C, packed)]
#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct SessionMessage {
    pub count: u32,
    pub length: u32,
}

pub(crate) struct PayloadMessage {
    _length: u32,
    pub data: Vec<u8>,
}

impl PayloadMessage {
    pub fn new(length: u32) -> Self {
        Self {
            _length: length,
            data: vec![0; length as usize],
        }
    }
}
