mod server;

use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use clap::Parser;
use flexi_logger::Logger;
use log::info;

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

    info!("Osiris version 0.1 - Torrent tracker");

    let args = Args::parse();
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), args.port);
    let udp_tracker = UdpTracker::new(addr).await?;
    info!("Listening on: {}", udp_tracker.socket.local_addr()?);

    udp_tracker.run().await?;

    Ok(())
}
