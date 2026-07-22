use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
const CHECK_TIMEOUT: Duration = Duration::from_millis(800);
pub async fn has_basic_connectivity(host: &str, tcp_port: u16) -> bool {
    if let Ok(ip) = host.parse::<IpAddr>() {
        let addr = SocketAddr::new(ip, tcp_port);
        if tcp_connect(addr).await {
            return true;
        }
    }
    let fallbacks = [
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53),
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53),
    ];
    for addr in fallbacks {
        if tcp_connect(addr).await {
            return true;
        }
    }
    false
}
async fn tcp_connect(addr: SocketAddr) -> bool {
    matches!(
        timeout(CHECK_TIMEOUT, TcpStream::connect(addr)).await,
        Ok(Ok(_))
    )
}
