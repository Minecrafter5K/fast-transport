use crate::{
    connection::{ Connection, ConnectionMeta },
    packet::Packet,
    socket::FastTransportSocket,
};

use std::sync::Arc;
use tokio::sync::{ mpsc, Mutex };

pub struct Endpoint {
    connections: Mutex<Vec<ConnectionMeta>>,
    incoming_rx: Mutex<mpsc::Receiver<Connection>>,
    incoming_tx: mpsc::Sender<Connection>,
}

impl Endpoint {
    pub fn new<T: FastTransportSocket + Send + 'static>(socket: T) -> Arc<Self> {
        let (incoming_tx, incoming_rx) = mpsc::channel::<Connection>(100);

        let endpoint = Endpoint {
            connections: Mutex::new(Vec::new()),
            incoming_rx: Mutex::new(incoming_rx),
            incoming_tx,
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
}

async fn handle_endpoint<T: FastTransportSocket>(socket: T, endpoint: Arc<Endpoint>) {
    loop {
        tokio::select! {
            packet = socket.receive_single() => handle_packet(packet, endpoint.clone()).await,
            // response = rx1_send.recv() => {
            //     fast_socket.send(response.unwrap(), client_address).await;
            // }
        }
    }
}

async fn handle_packet(packet: crate::packet::Packet, endpoint: Arc<Endpoint>) {
    match packet.get_id() {
        0 => handle_new_connection(packet, endpoint).await,
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
async fn handle_new_connection(packet: Packet, endpoint: Arc<Endpoint>) {
    let mut connections = endpoint.connections.lock().await;
    let connection_id = (connections.len() as u64) + 1;
    let (tx1, rx1): (mpsc::Sender<Packet>, mpsc::Receiver<Packet>) = mpsc::channel(20);
    let (tx2, rx2): (mpsc::Sender<Packet>, mpsc::Receiver<Packet>) = mpsc::channel(20);

    let connection_meta = ConnectionMeta::new(connection_id, tx2, rx1);
    connections.push(connection_meta);

    let connection = Connection::new(connection_id, tx1, rx2);
    endpoint.incoming_tx.send(connection).await.unwrap();
}

#[allow(dead_code)]
fn bytes_to_string(bytes: Vec<u8>) -> String {
    bytes
        .iter()
        .map(|&x| x as char)
        .collect()
}
