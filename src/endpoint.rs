use crate::{
    connection::{ Connection, ConnectionMeta },
    packet::Packet,
    socket::FastTransportSocket,
};

use std::{ net::SocketAddr, sync::Arc };
use tokio::sync::{ mpsc, Mutex };

pub struct Endpoint {
    connections: Mutex<Vec<ConnectionMeta>>,
    incoming_rx: Mutex<mpsc::Receiver<Connection>>,
    incoming_tx: mpsc::Sender<Connection>,
    outgoing_rx: Mutex<mpsc::Receiver<(Packet, SocketAddr)>>,
    outgoing_tx: mpsc::Sender<(Packet, SocketAddr)>,
}

impl Endpoint {
    pub fn new<T: FastTransportSocket + Send + 'static>(socket: T) -> Arc<Self> {
        let (incoming_tx, incoming_rx) = mpsc::channel::<Connection>(100);
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<(Packet, SocketAddr)>(100);

        let endpoint = Endpoint {
            connections: Mutex::new(Vec::new()),
            incoming_rx: Mutex::new(incoming_rx),
            incoming_tx,
            outgoing_rx: Mutex::new(outgoing_rx),
            outgoing_tx,
        };

        // TODO: kill task when endpoint is dropped
        let endpoint_arc = Arc::new(endpoint);
        tokio::spawn(handle_endpoint(socket, Arc::clone(&endpoint_arc)));
        endpoint_arc
    }

    pub async fn accept_connection(&self) -> Option<Connection> {
        let mut incoming_rx = self.incoming_rx.lock().await;
        incoming_rx.recv().await
    }

    pub async fn connect(&self, remote_addr: SocketAddr) -> Connection {
        let (tx, rx): (mpsc::Sender<Packet>, mpsc::Receiver<Packet>) = mpsc::channel(20);

        let connection_meta = ConnectionMeta::new(0, tx);
        let connection = Connection::new(0, remote_addr, rx, self.outgoing_tx.clone());

        let mut connections = self.connections.lock().await;
        connections.push(connection_meta);

        connection
    }
}

async fn handle_endpoint<T: FastTransportSocket>(socket: T, endpoint: Arc<Endpoint>) {
    loop {
        let mut outgoing_rx = endpoint.outgoing_rx.lock().await;
        tokio::select! {
            (packet, remote_addr) = socket.receive_single() => handle_packet(packet, remote_addr, endpoint.clone()).await,
            Some((packet, addr)) = outgoing_rx.recv() => {
                socket.send(packet, addr).await;
            }
        }
    }
}

async fn handle_packet(
    packet: crate::packet::Packet,
    remote_addr: SocketAddr,
    endpoint: Arc<Endpoint>
) {
    match packet.get_id() {
        0 => handle_new_connection(packet, remote_addr, endpoint).await,
        connection_id => {
            let connections = endpoint.connections.lock().await;
            let connection = connections.iter().find(|c| c.get_id() == connection_id);
            match connection {
                Some(connection) => {
                    let tx = connection.get_tx();
                    let send = tx.send(packet).await;
                    println!("Sent packet: {:?}", send);
                }
                None => println!("Connection not found"),
            }
        }
    };
}

#[allow(unused_variables)]
async fn handle_new_connection(packet: Packet, remote_addr: SocketAddr, endpoint: Arc<Endpoint>) {
    let mut connections = endpoint.connections.lock().await;
    let connection_id = (connections.len() as u64) + 1;
    let (tx, rx): (mpsc::Sender<Packet>, mpsc::Receiver<Packet>) = mpsc::channel(20);

    let connection_meta = ConnectionMeta::new(connection_id, tx);
    connections.push(connection_meta);

    let connection = Connection::new(connection_id, remote_addr, rx, endpoint.outgoing_tx.clone());
    endpoint.incoming_tx.send(connection).await.unwrap();
}

#[allow(dead_code)]
fn bytes_to_string(bytes: Vec<u8>) -> String {
    bytes
        .iter()
        .map(|&x| x as char)
        .collect()
}
