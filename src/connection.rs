use std::net::SocketAddr;

use anyhow::{ anyhow, Error };
use tokio::sync::mpsc;

use crate::{ packet::{ DataPayload, Packet, PacketHeader }, state::ConnectionState };

pub struct Connection {
    pub(crate) state: ConnectionState,
    connection_id: u32,
    remote_addr: SocketAddr,
    packet_tx: mpsc::Sender<(Packet, SocketAddr)>,
    packet_rx: mpsc::Receiver<Packet>,
}

impl Connection {
    pub(crate) fn new(
        connection_id: u32,
        remote_addr: SocketAddr,
        to_connection_rx: mpsc::Receiver<Packet>,
        from_connection_tx: mpsc::Sender<(Packet, SocketAddr)>
    ) -> Self {
        Connection {
            state: ConnectionState::Created,
            connection_id,
            remote_addr,
            packet_tx: from_connection_tx,
            packet_rx: to_connection_rx,
        }
    }

    pub async fn send(&self, buf: Vec<u8>) {
        let header = PacketHeader::new(0, 0, self.connection_id, 0);
        let packet = Packet::Data(header, DataPayload::new(buf));

        self.packet_tx.send((packet, self.remote_addr)).await.unwrap();
    }
    pub async fn recv(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let packet = self.packet_rx.recv().await.unwrap();
        let data = match &packet {
            Packet::Data(_, data) => data.get_data(),
            _ => panic!("unexpected packet type"),
        };
        buf.copy_from_slice(data);
        Ok(data.len())
    }

    pub(crate) async fn send_packet(&self, packet: Packet) -> Result<(), Error> {
        match self.packet_tx.send((packet, self.remote_addr)).await {
            Ok(_) => Ok(()),

            Err(e) => Err(anyhow!("Failed to send packet: {:?}", e)),
        }
    }
    pub(crate) async fn recv_packet(&mut self) -> Packet {
        self.packet_rx.recv().await.expect("recv_packet failed")
    }

    pub fn get_id(&self) -> u32 {
        self.connection_id
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        todo!("dropped connection: {}", self.connection_id);
    }
}

pub struct ConnectionMeta {
    connection_id: u32,
    to_connection_tx: mpsc::Sender<Packet>,
}

impl ConnectionMeta {
    pub fn new(connection_id: u32, to_connection_tx: mpsc::Sender<Packet>) -> Self {
        ConnectionMeta {
            connection_id,
            to_connection_tx,
        }
    }

    pub fn get_id(&self) -> u32 {
        self.connection_id
    }

    pub fn get_tx(&self) -> mpsc::Sender<Packet> {
        self.to_connection_tx.clone()
    }
}
