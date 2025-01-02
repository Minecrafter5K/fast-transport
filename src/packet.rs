#[allow(dead_code)]
#[derive(Debug, bincode::Decode, bincode::Encode)]
pub struct PacketHeader {
    version: u8,
    packet_type: u8,
    connection_id: u32,
    packet_number: u32,
}
impl PacketHeader {
    pub fn new(version: u8, packet_type: u8, connection_id: u32, packet_number: u32) -> Self {
        PacketHeader {
            version,
            packet_type,
            connection_id,
            packet_number,
        }
    }
}

#[derive(Debug, bincode::Decode, bincode::Encode, Default)]
pub struct ClientHelloPayload {}

#[derive(Debug, bincode::Decode, bincode::Encode)]
pub struct ServerHelloPayload {
    connection_id: u32,
}
impl ServerHelloPayload {
    pub fn get_connection_id(&self) -> u32 {
        self.connection_id
    }
}

#[derive(Debug, bincode::Decode, bincode::Encode)]
pub struct ConnAckPayload {}

#[derive(Debug, bincode::Decode, bincode::Encode)]
pub struct DataPayload {
    data: Vec<u8>,
}
impl DataPayload {
    pub fn new(data: Vec<u8>) -> Self {
        DataPayload { data }
    }
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, bincode::Decode, bincode::Encode)]
pub enum Packet {
    ClientHello(PacketHeader, ClientHelloPayload),
    ServerHello(PacketHeader, ServerHelloPayload),
    ConnAck(PacketHeader, ConnAckPayload),
    Data(PacketHeader, DataPayload),
}
impl Packet {
    pub fn new_client_hello() -> Self {
        Packet::ClientHello(PacketHeader::new(0, 2, 0, 0), ClientHelloPayload::default())
    }

    pub fn new_server_hello(connection_id: u32) -> Self {
        Packet::ServerHello(PacketHeader::new(0, 1, u32::MAX, 2), ServerHelloPayload {
            connection_id,
        })
    }

    pub fn new_ack(connection_id: u32) -> Self {
        Packet::ConnAck(PacketHeader::new(0, 3, connection_id, 3), ConnAckPayload {})
    }

    pub fn get_connection_id(&self) -> Option<u32> {
        match self {
            Packet::ClientHello(_, _) => None,
            Packet::ServerHello(_, payload) => Some(payload.connection_id),
            Packet::ConnAck(header, _) => Some(header.connection_id),
            Packet::Data(header, _) => Some(header.connection_id),
        }
    }
}
