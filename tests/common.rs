// use std::{net::{SocketAddr, IpAddr, Ipv4Addr}, error::Error};

// use flexi_logger::Logger;
// use log::info;
// use osiris::udp_server::server::UdpTracker;

// const DEFAULT_PORT: u16 = 6969;

// pub async fn setup() -> Result<(), Box<dyn Error>> {
//     Logger::try_with_str("debug")?.start()?;

//     let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), DEFAULT_PORT);
//     let udp_tracker = UdpTracker::new(addr).await?;
//     info!("Listening on: {}", udp_tracker.socket.local_addr()?);

//     udp_tracker.run().await?;

//     Ok(())
// }
