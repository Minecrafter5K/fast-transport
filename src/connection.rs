use tokio::sync::mpsc;

use crate::packet::Packet;

pub struct Connection {
    connection_id: u64,
    packet_tx: mpsc::Sender<Packet>,
    packet_rx: mpsc::Receiver<Packet>,
}

impl Connection {
    pub fn new(
        connection_id: u64,
        from_connection_tx: mpsc::Sender<Packet>,
        to_connection_rx: mpsc::Receiver<Packet>
    ) -> Self {
        Connection {
            connection_id,
            packet_tx: from_connection_tx,
            packet_rx: to_connection_rx,
        }
    }

    pub async fn send(&self, packet: Packet) {
        self.packet_tx.send(packet).await.unwrap();
    }
    pub async fn recv(&mut self) -> Packet {
        self.packet_rx.recv().await.unwrap()
    }

    pub fn get_id(&self) -> u64 {
        self.connection_id
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        todo!("dropped connection: {}", self.connection_id);
    }
}

pub struct ConnectionMeta {
    connection_id: u64,
    to_connection_tx: mpsc::Sender<Packet>,
    from_connection_rx: mpsc::Receiver<Packet>,
}

impl ConnectionMeta {
    pub fn new(
        connection_id: u64,
        to_connection_tx: mpsc::Sender<Packet>,
        from_connection_rx: mpsc::Receiver<Packet>
    ) -> Self {
        ConnectionMeta {
            connection_id,
            to_connection_tx,
            from_connection_rx,
        }
    }

    pub fn get_id(&self) -> u64 {
        self.connection_id
    }

    pub fn get_tx(&self) -> mpsc::Sender<Packet> {
        self.to_connection_tx.clone()
    }

    pub fn get_rx(&mut self) -> &mut mpsc::Receiver<Packet> {
        &mut self.from_connection_rx
    }
}
