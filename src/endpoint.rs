use crate::{
    connection::{ Connection, ConnectionMeta },
    packet::{ ClientHelloPayload, Packet, ServerHelloPayload },
    socket::FastTransportSocket,
    state::{ handshake_client, handshake_server },
};

use std::{ net::SocketAddr, sync::Arc };
use anyhow::Error;
use tokio::sync::{ mpsc, Mutex };

#[allow(dead_code)]
pub struct Endpoint {
    connections: Mutex<Vec<ConnectionMeta>>,
    incoming_rx: Mutex<mpsc::Receiver<Connection>>,
    incoming_tx: mpsc::Sender<Connection>,
    outgoing_rx: Mutex<mpsc::Receiver<(Packet, SocketAddr)>>,
    outgoing_tx: mpsc::Sender<(Packet, SocketAddr)>,
    new_connection_rx: Mutex<mpsc::Receiver<ServerHelloPayload>>,
    new_connection_tx: mpsc::Sender<ServerHelloPayload>,
}

impl Endpoint {
    pub fn new<T: FastTransportSocket + Send + 'static>(socket: T) -> Arc<Self> {
        let (incoming_tx, incoming_rx) = mpsc::channel::<Connection>(100);
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<(Packet, SocketAddr)>(100);
        let (new_connection_tx, new_connection_rx) = mpsc::channel::<ServerHelloPayload>(100);

        let endpoint = Endpoint {
            connections: Mutex::new(Vec::new()),
            incoming_rx: Mutex::new(incoming_rx),
            incoming_tx,
            outgoing_rx: Mutex::new(outgoing_rx),
            outgoing_tx,
            new_connection_rx: Mutex::new(new_connection_rx),
            new_connection_tx,
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

    pub async fn connect(&self, remote_addr: SocketAddr) -> Result<Connection, Error> {
        let (tx, rx): (mpsc::Sender<Packet>, mpsc::Receiver<Packet>) = mpsc::channel(20);

        let (connection, conn_id) = handshake_client(remote_addr, &self.new_connection_rx, (
            rx,
            self.outgoing_tx.clone(),
        )).await?;

        let connection_meta = ConnectionMeta::new(conn_id, tx);
        let mut connections = self.connections.lock().await;
        connections.push(connection_meta);

        Ok(connection)
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

#[allow(unused_variables)]
async fn handle_packet(packet: Packet, remote_addr: SocketAddr, endpoint: Arc<Endpoint>) {
    match packet {
        Packet::ClientHello(_, payload) => {
            tokio::spawn(handle_new_connection(payload, remote_addr, endpoint));
        }
        Packet::ServerHello(_, payload) => endpoint.new_connection_tx.send(payload).await.unwrap(),
        other_packet => {
            let conn_id = match other_packet.get_connection_id() {
                Some(id) => id,
                None => {
                    println!("Connection ID not found in packet");
                    return;
                }
            };

            let connections = endpoint.connections.lock().await;
            let connection = connections.iter().find(|c| c.get_id() == conn_id);
            match connection {
                None => println!("Connection not found"),
                Some(connection) => {
                    let tx = connection.get_tx();
                    tx.send(other_packet).await.expect("Failed to send packet to connection");
                }
            }
        }
    }
}

async fn handle_new_connection(
    payload: ClientHelloPayload,
    remote_addr: SocketAddr,
    endpoint: Arc<Endpoint>
) {
    let mut connections = endpoint.connections.lock().await;
    let connection_id = (connections.len() as u32) + 1;
    let (tx, rx): (mpsc::Sender<Packet>, mpsc::Receiver<Packet>) = mpsc::channel(20);

    let connection_meta = ConnectionMeta::new(connection_id, tx);
    connections.push(connection_meta);
    drop(connections);

    let mut connection = Connection::new(
        connection_id,
        remote_addr,
        rx,
        endpoint.outgoing_tx.clone()
    );
    handshake_server(&mut connection, payload, connection_id).await.unwrap();

    endpoint.incoming_tx.send(connection).await.unwrap();
}

#[allow(dead_code)]
fn bytes_to_string(bytes: Vec<u8>) -> String {
    bytes
        .iter()
        .map(|&x| x as char)
        .collect()
}
