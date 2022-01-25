# getrandomport

Get a random not-in-use port.

```rust
let port = reserve_udp_port();
UdpSocket::bind(format!("127.0.0.1:{}", port.take())).unwrap();

let port = reserve_tcp_port();
TcpListener::bind(format!("127.0.0.1:{}", port.take())).unwrap();
```
