mod server;

use std::{error::Error, net::{SocketAddr, IpAddr, Ipv4Addr}};

use clap::Parser;
use flexi_logger::Logger;
use log::info;
use tokio::net::UdpSocket;

use crate::server::UdpTracker;

/// Osiris torrent tracker
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Tracker port
    #[clap(short, long, value_parser, default_value_t = 6969)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Logger::try_with_str("debug")?.start()?;

    let args = Args::parse();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), args.port);
    let socket = UdpSocket::bind(addr).await?;

    info!("Listening on: {}", socket.local_addr()?);

    let udp_tracker = UdpTracker::new(socket)?;
    udp_tracker.process().await?;

    Ok(())
}
