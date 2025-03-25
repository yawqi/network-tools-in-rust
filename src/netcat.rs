use anyhow::Ok;
use clap::clap_derive::Parser;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{tcp::{OwnedReadHalf, OwnedWriteHalf}, TcpListener, TcpStream}, task::JoinHandle};
use tracing::{error, info, instrument};

#[derive(Parser, Debug)]
#[command(version, about, long_about)]
pub struct NetcatArgs {
    #[clap(short, long, group = "connection_type")]
    listen_port: Option<u16>,
    #[clap(short, long, group = "connection_type", requires = "port")]
    server: Option<String>,
    #[clap(short, long, value_name = "port to connect", requires = "server")]
    port: Option<u16>,

    #[allow(unused)]
    #[clap(skip)]
    connection_type: String,
}

#[instrument]
pub async fn run(args: NetcatArgs) -> Result<(), anyhow::Error> {
    if args.listen_port.is_some() {
        run_server(args.listen_port.unwrap()).await?;
    } else if args.server.is_some() && args.port.is_some() {
        run_client(args.server.unwrap(), args.port.unwrap()).await?;
    } else {
        error!("Must specify either --listen-port or --server");
        return Err(anyhow::anyhow!("Invalid arguments"));
    }

    Ok(())
}

#[instrument]
async fn run_server(port: u16) -> Result<(), anyhow::Error> {
    let listener = TcpListener::bind("localhost:".to_string() + &port.to_string()).await?;
    loop {
        let (client, caddr) = listener.accept().await?;
        info!(client_address = %caddr, "Accepted connection:");
        tokio::spawn(handle_client(client));
    }
}

#[instrument]
async fn run_client(server: String, port: u16) -> Result<(), anyhow::Error> {
    let client = TcpStream::connect(server + ":" + &port.to_string()).await?;
    let (reader, writer) = client.into_split();
    tokio::select! {
        _ = handle_reader(reader) => {},
        _ = handle_writer(writer) => {},
    }
    Ok(())
}

#[instrument]
async fn handle_client(client: TcpStream) -> Result<(), anyhow::Error> {
    let (reader, writer) = client.into_split();
    tokio::try_join!(handle_reader(reader), handle_writer(writer))?;

    Ok(())
}

async fn handle_reader(mut reader: OwnedReadHalf) -> Result<(), anyhow::Error> {
    let (tx_socket, mut rx_stdout) = tokio::sync::mpsc::channel(1024);
    let h1: JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async move {
        let mut buf = vec![0u8; 1024];
        let mut total_bytes = 0;

        let mut now = tokio::time::Instant::now();
        loop {
            let n = reader.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            total_bytes += n;
            if tokio::time::Instant::now() - now >= tokio::time::Duration::from_secs(1) {
                info!("throughput = {:.2} MB/s", total_bytes as f64 / 1024.0 / 1024.0);
                total_bytes = 0;
                now = tokio::time::Instant::now();
            }

            tx_socket.send(buf[..n].to_vec()).await?;
        }
        Ok(())
    });

    let h2 : JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async move {
        while let Some(n) = rx_stdout.recv().await {
            tokio::io::stdout().write_all(&n).await?;
        }
        Ok(())
    });

    h1.await??;
    h2.await??;

    Ok(())
}

async fn handle_writer(mut writer: OwnedWriteHalf) -> Result<(), anyhow::Error> {
    let (tx_stdin, mut rx_socket) = tokio::sync::mpsc::channel(1024);
    let h1 : JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async move {
        let mut buf = vec![0u8; 1024];
        loop {
            let n = tokio::io::stdin().read(&mut buf).await?;
            if n == 0 {
                break;
            }
            tx_stdin.send(buf[..n].to_vec()).await?;
        }
        Ok(())
    });

    let h2 : JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async move {
        while let Some(n) = rx_socket.recv().await {
            writer.write_all(&n).await?;
        }
        Ok(())
    });

    h1.await??;
    h2.await??;

    Ok(())
}

