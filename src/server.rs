use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::SystemTime,
};

use bincode::{
    self,
    config::{
        AllowTrailing, BigEndian, FixintEncoding, WithOtherEndian, WithOtherIntEncoding,
        WithOtherTrailing,
    },
    DefaultOptions, Options,
};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;

// Magic constant from https://www.bittorrent.org/beps/bep_0015.html
const PROTOCOL_ID: u64 = 0x41727101980;
const DEFAULT_ANNOUNCE_INTERVAL: u32 = 10 * 60 * 1000; // 10 minutes

#[repr(u32)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum TrackerAction {
    Connect = 0,
    Announce = 1,
    Scrape = 2,
    Error = 3,
}

#[repr(u32)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum TrackerEvent {
    None = 0,
    Completed = 1,
    Started = 2,
    Stopped = 3,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TrackerPacketHeader {
    connection_id: u64,
    action: TrackerAction,
    transaction_id: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TrackerConnectResponsePacket {
    action: TrackerAction,
    transaction_id: u32,
    connection_id: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TrackerAnnounceRequestPacket {
    header: TrackerPacketHeader,
    info_hash: [u8; 20],
    peer_id: [u8; 20],
    downloaded: u64,
    left: u64,
    uploaded: u64,
    event: TrackerEvent,
    ip_address: u32,
    key: u32,
    num_want: u32,
    port: u16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TrackerAnnounceResponsePacket {
    action: TrackerAction,
    transaction_id: u32,
    interval: u32,
    leechers: u32,
    seeders: u32,
    ip_address: u32,
    port: u16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TrackerScrapeRequestPacket {
    header: TrackerPacketHeader,
    info_hash: [u8; 20],
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TrackerScrapeResponsePacket {
    action: TrackerAction,
    transaction_id: u32,
    info_hash: [u8; 20],
    seeders: u32,
    completed: u32,
    leechers: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TrackerErrorResponsePacket {
    action: TrackerAction,
    transaction_id: u32,
    error_string: String,
}

pub struct UdpTracker {
    // TODO: There must be a better way to store the bincode config
    bincode_config: WithOtherTrailing<
        WithOtherIntEncoding<WithOtherEndian<DefaultOptions, BigEndian>, FixintEncoding>,
        AllowTrailing,
    >,
    pub socket: UdpSocket,
}

impl UdpTracker {
    pub async fn new(addr: SocketAddr) -> Result<UdpTracker, io::Error> {
        let bincode_config = bincode::DefaultOptions::new()
            .with_big_endian()
            .with_fixint_encoding()
            .allow_trailing_bytes();
        let socket = UdpSocket::bind(addr).await?;

        Ok(UdpTracker {
            bincode_config,
            socket,
        })
    }

    pub async fn run(self) -> Result<(), io::Error> {
        let udp_tracker = Arc::new(self);

        loop {
            let mut buffer = [0; 1024];
            let udp_tracker = udp_tracker.clone();

            // TODO: Figure out if channels are needed
            // let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);

            let (len, addr) = udp_tracker.socket.recv_from(&mut buffer).await?;
            debug!("{:?} bytes received from {:?}", len, addr);

            tokio::spawn(async move {
                udp_tracker.process(&addr, &buffer).await.unwrap();
            });
        }
    }

    pub async fn process(&self, addr: &SocketAddr, packet: &[u8; 1024]) -> Result<(), io::Error> {
        match self
            .bincode_config
            .deserialize::<TrackerPacketHeader>(packet)
        {
            Ok(tracker_packet_header) => match tracker_packet_header.action {
                TrackerAction::Connect => self.handle_connect(addr, tracker_packet_header).await?,
                TrackerAction::Announce => {
                    self.handle_announce(addr, tracker_packet_header, packet)
                        .await?
                }
                TrackerAction::Scrape => {
                    self.handle_scrape(addr, tracker_packet_header, packet)
                        .await?
                }
                _ => unimplemented!(),
            },
            Err(_) => error!("Invalid packet"),
        }

        Ok(())
    }

    async fn handle_connect(
        &self,
        addr: &SocketAddr,
        tracker_packet_header: TrackerPacketHeader,
    ) -> Result<(), io::Error> {
        debug!("{:?}", tracker_packet_header);

        let response: Vec<u8> = if tracker_packet_header.connection_id != PROTOCOL_ID {
            error!(
                "Error: {:?} is not a valid protocol ID",
                tracker_packet_header.connection_id
            );
            let tracker_error_response_packet = TrackerErrorResponsePacket {
                action: TrackerAction::Error,
                transaction_id: tracker_packet_header.transaction_id,
                error_string: String::from("Not a valid protocol ID"),
            };
            debug!("{:?}", tracker_error_response_packet);

            self.bincode_config
                .serialize(&tracker_error_response_packet)
                .unwrap()
        } else {
            let tracker_connect_response_packet = TrackerConnectResponsePacket {
                action: TrackerAction::Connect,
                transaction_id: tracker_packet_header.transaction_id,
                connection_id: generate_connection_id(addr),
            };
            debug!("{:?}", tracker_connect_response_packet);

            self.bincode_config
                .serialize(&tracker_connect_response_packet)
                .unwrap()
        };

        let len = self.socket.send_to(&response, addr).await?;
        debug!("{:?} bytes sent: {:?}", len, response);

        Ok(())
    }

    async fn handle_announce(
        &self,
        addr: &SocketAddr,
        tracker_packet_header: TrackerPacketHeader,
        payload: &[u8; 1024],
    ) -> Result<(), io::Error> {
        debug!("{:?}", tracker_packet_header);

        let response: Vec<u8> = match self
            .bincode_config
            .deserialize::<TrackerAnnounceRequestPacket>(payload)
        {
            Ok(tracker_announce_request_packet) => {
                debug!("{:?}", tracker_announce_request_packet);

                let mut ip_address: u32 = 0;
                if let IpAddr::V4(ipv4) = addr.ip() {
                    ip_address = ipv4.into();
                }

                let tracker_announce_response_packet = TrackerAnnounceResponsePacket {
                    action: TrackerAction::Announce,
                    transaction_id: tracker_announce_request_packet.header.transaction_id,
                    interval: DEFAULT_ANNOUNCE_INTERVAL,
                    leechers: 0,
                    seeders: 0,
                    ip_address,
                    port: addr.port(),
                };
                debug!("{:?}", tracker_announce_response_packet);
                debug!(
                    "IP: {:?} - Info hash: {:?} - Peer id: {:?}",
                    IpAddr::V4(Ipv4Addr::from(tracker_announce_response_packet.ip_address)),
                    String::from_utf8_lossy(&tracker_announce_request_packet.info_hash),
                    String::from_utf8_lossy(&tracker_announce_request_packet.peer_id),
                );

                self.bincode_config
                    .serialize(&tracker_announce_response_packet)
                    .unwrap()
            }
            Err(_) => {
                error!("Announce error");

                let tracker_error_response_packet = TrackerErrorResponsePacket {
                    action: TrackerAction::Error,
                    transaction_id: tracker_packet_header.transaction_id,
                    error_string: String::from("Announce error"),
                };
                debug!("{:?}", tracker_error_response_packet);

                self.bincode_config
                    .serialize(&tracker_error_response_packet)
                    .unwrap()
            }
        };

        let len = self.socket.send_to(&response, addr).await?;
        debug!("{:?} bytes sent: {:?}", len, response);

        Ok(())
    }

    async fn handle_scrape(
        &self,
        addr: &SocketAddr,
        tracker_packet_header: TrackerPacketHeader,
        payload: &[u8; 1024],
    ) -> Result<(), io::Error> {
        debug!("{:?}", tracker_packet_header);

        let response: Vec<u8> = match self
            .bincode_config
            .deserialize::<TrackerScrapeRequestPacket>(payload)
        {
            Ok(tracker_scrape_request_packet) => {
                let info_hash: [u8; 20] = [
                    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                    0x0, 0x0, 0x0, 0x0,
                ];
                debug!("{:?}", tracker_scrape_request_packet);

                let tracker_scrape_response_packet = TrackerScrapeResponsePacket {
                    action: TrackerAction::Announce,
                    transaction_id: tracker_scrape_request_packet.header.transaction_id,
                    info_hash,
                    seeders: 0,
                    completed: 0,
                    leechers: 0,
                };
                debug!("{:?}", tracker_scrape_response_packet);

                self.bincode_config
                    .serialize(&tracker_scrape_response_packet)
                    .unwrap()
            }
            Err(_) => {
                error!("Scrape error");

                let tracker_error_response_packet = TrackerErrorResponsePacket {
                    action: TrackerAction::Error,
                    transaction_id: tracker_packet_header.transaction_id,
                    error_string: String::from("Scrape error"),
                };
                debug!("{:?}", tracker_error_response_packet);

                self.bincode_config
                    .serialize(&tracker_error_response_packet)
                    .unwrap()
            }
        };

        let len = self.socket.send_to(&response, addr).await?;
        debug!("{:?} bytes sent: {:?}", len, response);

        Ok(())
    }
}

fn generate_connection_id(addr: &SocketAddr) -> u64 {
    debug!("{:?} - {:?}", rand::random::<u32>(), addr.ip());
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs(),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use bincode::Options;

    use crate::server::{
        generate_connection_id, TrackerConnectResponsePacket, TrackerPacketHeader, PROTOCOL_ID,
    };

    #[test]
    fn it_deserialize_connect_request() {
        let options = bincode::DefaultOptions::new()
            .with_big_endian()
            .with_fixint_encoding()
            .allow_trailing_bytes();

        let buffer: [u8; 16] = [
            0x0, 0x0, 0x4, 0x17, 0x27, 0x10, 0x19, 0x80, // protocol id: 0x41727101980
            0x0, 0x0, 0x0, 0x0, // action: 0
            0x0, 0x0, 0x0, 0x2A, // transaction id: 42
        ];

        let tracker_packet_header: TrackerPacketHeader = options.deserialize(&buffer).unwrap();

        assert_eq!(
            tracker_packet_header,
            TrackerPacketHeader {
                connection_id: PROTOCOL_ID,
                action: 0,
                transaction_id: 42,
            }
        );
    }

    #[test]
    fn it_serialize_connect_response() {
        let options = bincode::DefaultOptions::new()
            .with_big_endian()
            .with_fixint_encoding()
            .allow_trailing_bytes();

        let tracker_connect_response_packet = TrackerConnectResponsePacket {
            action: 0,
            transaction_id: 1,
            connection_id: 1,
        };
        let expected: [u8; 16] = [
            0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, // connection id: 1
            0x0, 0x0, 0x0, 0x0, // action: 0
            0x0, 0x0, 0x0, 0x1, // transaction id: 1
        ];

        let response: Vec<u8> = options.serialize(&tracker_connect_response_packet).unwrap();

        assert_eq!(response, expected);
    }

    #[test]
    fn it_generates_connection_id() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        println!("{:?}", generate_connection_id(addr));
    }
}
