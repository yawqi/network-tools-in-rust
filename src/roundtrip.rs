use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use clap::Parser;
use chrono::Local;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{event, Level};

use std::net::{TcpListener, TcpStream};

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
pub struct RoundTripArgs {
    #[clap(short, long)]
    server: bool,
    #[clap(short = 'a', long, value_name = "Server Addr", default_value = "127.0.0.1")]
    host: Option<String>,
    #[clap(short, long, value_name = "port number", default_value = "12345")]
    port: Option<u16>,
}

impl RoundTripArgs {
    pub fn is_server(&self) -> bool {
        self.server
    }

    pub fn host(&self) -> &str {
        self.host.as_ref().unwrap()
    }

    pub fn port(&self) -> u16 {
        self.port.unwrap()
    }
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
struct RTTFrame {
    t1: i64,
    t2: i64,
}

impl RTTFrame {
    async fn async_receive(async_stream: &mut tokio::net::TcpStream) -> anyhow::Result<Self> {
        let t1 = async_stream.read_i64().await?;
        let t2 = async_stream.read_i64().await?;
        Ok(Self {
            t1,
            t2,
        })
    }

    fn receive(stream: &mut TcpStream) -> std::io::Result<Self> {
        let t1 = stream.read_i64::<NetworkEndian>()?;
        let t2 = stream.read_i64::<NetworkEndian>()?;
        Ok(Self {
            t1,
            t2,
        })
    }

    async fn async_send(self, stream: &mut tokio::net::TcpStream) -> anyhow::Result<()> {
        stream.write_i64(self.t1).await?;
        stream.write_i64(self.t2).await?;
        Ok(())
    }

    fn send(self, stream: &mut TcpStream) -> std::io::Result<()> {
        stream.write_i64::<NetworkEndian>(self.t1)?;
        stream.write_i64::<NetworkEndian>(self.t2)?;
        Ok(())
    }

    fn set_t1_to_now(&mut self) {
        self.t1 = Local::now().timestamp_micros();
    }

    fn set_t2_to_now(&mut self) {
        self.t2 = Local::now().timestamp_micros();
    }
}

pub fn server(host: &str, port: u16) -> anyhow::Result<()> {
    let addr = host.to_string() + ":" + &port.to_string();
    let listener = TcpListener::bind(addr)?;

    loop {
        let (mut client, _client_addr) = listener.accept()?;
        std::thread::spawn(move || {
            event!(Level::INFO, "handling client: {:?}", _client_addr);
            loop {
               match RTTFrame::receive(&mut client) {
                    Ok(mut frame) => {
                        frame.set_t2_to_now();
                        let _ = frame.send(&mut client);
                    },
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::UnexpectedEof {
                            event!(Level::INFO, "Client disconnected");
                            break;
                        }
                    }
                } 
            }
        });
    }
}

pub fn client(host: &str, port: u16) -> anyhow::Result<()> {
    let addr = host.to_string() + ":" + &port.to_string();
    let mut stream = TcpStream::connect(addr)?;

    loop {
        let mut frame = RTTFrame::default();
        frame.set_t1_to_now();
        frame.send(&mut stream)?;
        let frame = RTTFrame::receive(&mut stream)?;
        let t3 = Local::now().timestamp_micros();

        let rtt = t3 - frame.t1;
        let diff = frame.t2 - (t3 + frame.t1) / 2;
        event!(Level::INFO, "RTT: {} us, diff: {} us", rtt, diff);
    }
}

pub async fn async_server(host: &str, port: u16) -> anyhow::Result<()> {
    let addr = host.to_string() + ":" + &port.to_string();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    loop {
        let (mut client, client_addr) = listener.accept().await?;
        event!(Level::INFO, "handling client: {:?}", client_addr);
        let _ = tokio::spawn(async move {
            loop {
                if let Ok(frame) = RTTFrame::async_receive(&mut client).await {
                    let mut frame = frame;
                    frame.set_t2_to_now();
                    let _ = frame.async_send(&mut client).await;
                } else {
                    event!(Level::INFO, "Client disconnected");
                    break;
                }
            }
        });
    }
}

pub async fn async_client(host: &str, port: u16) -> anyhow::Result<()> {
    let addr = host.to_string() + ":" + &port.to_string();
    let mut stream = tokio::net::TcpStream::connect(addr).await?;
    event!(Level::ERROR, "Connected to server");
    loop {
        let mut frame = RTTFrame::default();
        frame.set_t1_to_now();
        frame.async_send(&mut stream).await?;
        let frame = RTTFrame::async_receive(&mut stream).await?;
        let t3 = Local::now().timestamp_micros();
        let rtt = t3 - frame.t1;
        let diff = frame.t2 - (t3 + frame.t1) / 2;
        event!(Level::INFO, "RTT: {} us, diff: {} us", rtt, diff);
    }
}