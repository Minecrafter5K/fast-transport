#[derive(Debug)]
pub struct Packet {
    id: u64,
    data: Vec<u8>,
}

impl Packet {
    pub fn new(data: Vec<u8>, id: Option<u64>) -> Self {
        Packet {
            id: id.unwrap_or(0),
            data,
        }
    }

    pub fn into_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(self.id.to_be_bytes().iter());
        bytes.extend(self.data);
        bytes
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        Packet {
            id: u64::from_be_bytes(data[0..8].try_into().unwrap()),
            data: data[8..].to_vec(),
        }
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn get_id(&self) -> u64 {
        self.id
    }
}
