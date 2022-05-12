# Getport

Port reservation helper.

```rust
let port = reserve_udp_port();
UdpSocket::bind(format!("127.0.0.1:{}", port.take())).unwrap();

let port = reserve_tcp_port();
TcpListener::bind(format!("127.0.0.1:{}", port.take())).unwrap();
```

## Install

```toml
getport = { git = "https://github.com/kafji/getport", tag = "v0.3.0" }
```
