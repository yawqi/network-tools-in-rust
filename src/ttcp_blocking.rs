use crate::common::{Args, PayloadMessage, SessionMessage};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    time::Instant,
};

use anyhow::anyhow;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};

fn write_session_message(
    stream: &mut TcpStream,
    count: u32,
    length: u32,
) -> Result<(), anyhow::Error> {
    stream.write_u32::<NetworkEndian>(count)?;
    stream.write_u32::<NetworkEndian>(length)?;
    Ok(())
}

pub fn transmit(args: Args) -> Result<(), anyhow::Error> {
    let addr = args
        .get_server_address()
        .ok_or(anyhow!("No server address specified"))?;
    let port = args.get_port();
    let count = args.get_packet_count();
    let length = args.get_packet_size();

    let mut stream = TcpStream::connect(addr.to_string() + ":" + &port.to_string())?;
    // stream.set_nodelay(true)?;
    write_session_message(&mut stream, count, length)?;

    (0..count).try_for_each(|_| -> Result<(), anyhow::Error> {
        let payload = PayloadMessage::new(length);
        stream.write_u32::<NetworkEndian>(length)?;
        stream.write_all(&payload.data)?;

        let _ = stream.read_u32::<NetworkEndian>()?;
        Ok(())
    })?;

    Ok(())
}

fn read_session_message(stream: &mut TcpStream) -> Result<SessionMessage, anyhow::Error> {
    let mut session_message = SessionMessage::default();
    session_message.count = stream.read_u32::<NetworkEndian>()?;
    session_message.length = stream.read_u32::<NetworkEndian>()?;

    Ok(session_message)
}

pub fn receive(args: Args) -> Result<(), anyhow::Error> {
    let addr = args
        .get_server_address()
        .ok_or(anyhow!("No server address specified"))?;
    let port = args.get_port();

    let listener = TcpListener::bind(addr.to_string() + ":" + &port.to_string())?;
    let (mut stream, _client) = listener.accept()?;
    dbg!(_client);
    let SessionMessage { count, length } = read_session_message(&mut stream)?;

    let now = Instant::now();
    // dbg!(count, length);
    (0..count).try_for_each(|_| -> Result<(), anyhow::Error> {
        let payload_length = stream.read_u32::<NetworkEndian>()?;
        assert!(payload_length == length);

        let mut payload = PayloadMessage::new(payload_length);
        stream.read_exact(&mut payload.data)?;
        stream.write_u32::<NetworkEndian>(payload_length)?;
        Ok(())
    })?;
    let elapsed = now.elapsed();
    let throughput = (count as f64 * length as f64 ) as f64 / elapsed.as_secs_f64();
    println!(
        "Received {} packets ({} bytes) in {:.2?} ({:.2} Gbps)",
        count,
        count as f64 * length as f64,
        elapsed,
        throughput * 8.0 / 1e9
    );

    Ok(())
}
