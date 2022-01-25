use getrandomport::{reserve_tcp_port, reserve_udp_port};
use std::net::{TcpListener, UdpSocket};

#[test]
fn test_sanity_for_udp() {
    let port = reserve_udp_port().peek();
    assert!(49152 <= port);
}

#[test]
fn test_sanity_for_tcp() {
    let port = reserve_tcp_port().peek();
    assert!(49152 <= port);
}

#[test]
fn test_port_is_asref_u16() {
    let port = reserve_udp_port();
    let _: &dyn AsRef<u16> = &port;
    assert_eq!(*port.as_ref(), port.peek())
}

#[test]
fn test_basic_usage_scenario_for_udp() {
    let port = reserve_udp_port();
    UdpSocket::bind(format!("127.0.0.1:{}", port.take())).unwrap();
}

#[test]
fn test_basic_usage_scenario_for_tcp() {
    let port = reserve_tcp_port();
    TcpListener::bind(format!("127.0.0.1:{}", port.take())).unwrap();
}
