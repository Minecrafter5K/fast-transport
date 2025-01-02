use std::result::Result::Ok;
use std::net::SocketAddr;

use anyhow::{ anyhow, Error };
use tokio::sync::{ mpsc, Mutex };

use crate::{ connection::Connection, packet::{ ClientHelloPayload, Packet, ServerHelloPayload } };

pub(crate) enum ConnectionState {
    Created,
    Handshaking,
    Connected,
}

pub(crate) async fn handshake_server(
    conn: &mut Connection,
    _client_hello: ClientHelloPayload,
    conn_id: u32
) -> Result<(), Error> {
    conn.state = ConnectionState::Handshaking;

    conn.send_packet(Packet::new_server_hello(conn_id)).await.unwrap();

    let ack: Packet = conn.recv_packet().await;
    println!("Received ConnAck packet: {:?}", ack.get_connection_id());

    conn.state = ConnectionState::Connected;
    Ok(())
}

pub(crate) async fn handshake_client(
    remote_addr: SocketAddr,
    new_connection_packet_rx: &Mutex<mpsc::Receiver<ServerHelloPayload>>,
    (to_connection_rx, from_connection_tx): (
        mpsc::Receiver<Packet>,
        mpsc::Sender<(Packet, SocketAddr)>,
    )
) -> Result<(Connection, u32), Error> {
    let mut new_connection_rx = new_connection_packet_rx.lock().await;

    let client_hello = (Packet::new_client_hello(), remote_addr);
    match from_connection_tx.send(client_hello).await {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Failed to send ClientHello packet: {:?}", e));
        }
    }

    let server_hello = new_connection_rx.recv().await.unwrap();
    let conn_id = server_hello.get_connection_id();
    println!("Received ServerHello packet for new connection ID: {:?}", conn_id);

    // let server_data = Packet::
    let mut conn = Connection::new(conn_id, remote_addr, to_connection_rx, from_connection_tx);

    match conn.send_packet(Packet::new_ack(conn_id)).await {
        Ok(_) => (),
        Err(e) => {
            return Err(anyhow!("Failed to send Ack packet: {:?}", e));
        }
    }

    conn.state = ConnectionState::Connected;

    Ok((conn, conn_id))
}
