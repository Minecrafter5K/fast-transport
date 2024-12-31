use crate::packet::Packet;

pub trait FastTransportSocket {
    fn send(
        &self,
        data: Packet,
        addr: std::net::SocketAddr
    ) -> impl std::future::Future<Output = ()> + Send;
    fn receive_single(&self) -> impl std::future::Future<Output = Packet> + Send;
}
