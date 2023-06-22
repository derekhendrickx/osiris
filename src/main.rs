mod udp_server;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use clap::Parser;
use flexi_logger::Logger;
use log::{info, debug};
use tokio::net::UdpSocket;

/// Osiris torrent tracker
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Tracker port
    #[clap(short, long, value_parser, default_value_t = 6969)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Logger::try_with_str("debug")?.start()?;

    info!("Osiris version 0.1 - Torrent tracker");

    let args = Args::parse();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), args.port);
    let socket = UdpSocket::bind(addr).await?;

    info!("Listening on {:?}:{:?}", addr.ip(), addr.port());

    let mut buffer = [0; 1024];
    loop {
        let (len, addr) = socket.recv_from(&mut buffer).await?;
        debug!("{:?} bytes received from {:?}", len, addr);

        let len = socket.send_to(&buffer[..len], addr).await?;
        debug!("{:?} bytes sent", len);
    }
}
