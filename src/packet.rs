use anyhow::Error;

#[allow(dead_code)]
#[derive(Debug)]
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
impl PacketHeader {
    pub fn into_bytes(&self) -> [u8; 13] {
        let mut bytes = [0; 13];
        bytes[0] = self.version;
        bytes[1] = self.packet_type;
        bytes[2..4].copy_from_slice(&[0, 0]);
        bytes[4..8].copy_from_slice(&self.connection_id.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.packet_number.to_be_bytes());
        bytes
    }
    pub fn from_bytes(bytes: &[u8]) -> Self {
        PacketHeader {
            version: bytes[0],
            packet_type: bytes[1],
            connection_id: u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
            packet_number: u32::from_be_bytes(bytes[8..12].try_into().unwrap()),
        }
    }
}

#[derive(Debug, Default)]
pub struct ClientHelloPayload {}

#[derive(Debug)]
pub struct ServerHelloPayload {
    connection_id: u32,
}
impl ServerHelloPayload {
    pub fn get_connection_id(&self) -> u32 {
        self.connection_id
    }
}

#[derive(Debug)]
pub struct ConnAckPayload {}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum Packet {
    ClientHello(PacketHeader, ClientHelloPayload),
    ServerHello(PacketHeader, ServerHelloPayload),
    ConnAck(PacketHeader, ConnAckPayload),
    Data(PacketHeader, DataPayload),
}
impl Packet {
    pub fn new_client_hello() -> Self {
        Packet::ClientHello(PacketHeader::new(0, 1, 0, 0), ClientHelloPayload::default())
    }

    pub fn new_server_hello(connection_id: u32) -> Self {
        Packet::ServerHello(PacketHeader::new(0, 2, u32::MAX, 2), ServerHelloPayload {
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

    pub fn into_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Packet::ClientHello(header, _) => {
                bytes.extend_from_slice(&header.into_bytes());
            }
            Packet::ServerHello(header, payload) => {
                bytes.extend_from_slice(&header.into_bytes());
                bytes.extend_from_slice(&payload.connection_id.to_be_bytes());
            }
            Packet::ConnAck(header, _) => {
                bytes.extend_from_slice(&header.into_bytes());
            }
            Packet::Data(header, payload) => {
                bytes.extend_from_slice(&header.into_bytes());
                bytes.extend_from_slice(&payload.data);
            }
        }
        bytes
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let header = PacketHeader::from_bytes(&bytes[..13]);
        match header.packet_type {
            0 => Ok(Packet::Data(header, DataPayload::new(bytes[13..].to_vec()))),
            1 => Ok(Packet::ClientHello(header, ClientHelloPayload {})),
            2 => {
                println!("ServerHello: {:?}", bytes);
                Ok(
                    Packet::ServerHello(header, ServerHelloPayload {
                        connection_id: u32::from_be_bytes(bytes[13..17].try_into().unwrap()),
                    })
                )
            }
            3 => Ok(Packet::ConnAck(header, ConnAckPayload {})),
            _ => Err(anyhow::anyhow!("Invalid packet type")),
        }
    }
}
