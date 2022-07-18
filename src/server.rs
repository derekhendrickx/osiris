use std::{io, net::SocketAddr, sync::Arc, time::SystemTime};

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

#[repr(u32)]
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum TrackerAction {
    Connect = 0,
    Announce = 1,
    Scrape = 2,
    Error = 3,
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
                TrackerAction::Announce => todo!(),
                TrackerAction::Scrape => todo!(),
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
