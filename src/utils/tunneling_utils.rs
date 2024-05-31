use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use wiretun::Tun;
use wiretun::TunError;

#[derive(Clone)]
pub struct StubTun {
    tx: mpsc::Sender<Vec<u8>>,
    rx: Arc<Mutex<mpsc::Receiver<Vec<u8>>>>,
}

impl StubTun {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(128);
        let rx = Arc::new(Mutex::new(rx));
        Self { tx, rx }
    }

    pub fn handle(&self, mut buf: Vec<u8>) -> Vec<u8> {
        use pnet::packet::ip::IpNextHeaderProtocols;
        use pnet::packet::ipv4::{checksum, MutableIpv4Packet};
        use pnet::packet::udp::{ipv4_checksum, MutableUdpPacket};
        use pnet::packet::Packet;
        let mut ipv4 = MutableIpv4Packet::new(&mut buf).unwrap();
        let src_ip = ipv4.get_source();
        let dst_ip = ipv4.get_destination();
        ipv4.set_source(dst_ip);
        ipv4.set_destination(src_ip);

        match ipv4.get_next_level_protocol() {
            IpNextHeaderProtocols::Udp => {
                let mut udp = MutableUdpPacket::owned(ipv4.payload().to_vec()).unwrap();
                let src_port = udp.get_source();
                let dst_port = udp.get_destination();
                udp.set_source(dst_port);
                udp.set_destination(src_port);
                udp.set_checksum(ipv4_checksum(&udp.to_immutable(), &dst_ip, &src_ip));
                ipv4.set_payload(udp.packet());
            }
            _ => {
                debug!("Unknown packet type!");
            }
        }

        ipv4.set_checksum(checksum(&ipv4.to_immutable()));

        ipv4.packet().to_vec()
    }
}

#[async_trait]
impl Tun for StubTun {
    fn name(&self) -> &str {
        "stub"
    }

    fn mtu(&self) -> Result<u16, TunError> {
        Ok(1500)
    }

    fn set_mtu(&self, _mtu: u16) -> Result<(), TunError> {
        Ok(())
    }

    async fn recv(&self) -> Result<Vec<u8>, TunError> {
        let mut rx = self.rx.lock().await;
        let rv = rx.recv().await.ok_or(TunError::Closed);

        match &rv {
            Ok(buf) => {
                info!("recv data[{}] from tun", buf.len());
            }
            Err(e) => {
                error!("failed to recv data from tun: {:?}", e);
            }
        }

        rv
    }

    async fn send(&self, buf: &[u8]) -> Result<(), TunError> {
        info!("recv data[{}] from outbound", buf.len());
        self.tx
            .send(self.handle(buf.to_vec()))
            .await
            .map_err(|_| TunError::Closed)
    }
}
