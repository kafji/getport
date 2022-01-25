use std::{
    io::ErrorKind,
    net::{SocketAddr, TcpListener, UdpSocket},
};

/// Port reservation.
pub struct Port<T> {
    number: u16,
    _sitter: T,
}

impl<T> Port<T> {
    /// Take the port number & release its sitter.
    pub fn take(self) -> u16 {
        self.number
    }

    /// Returns the port number without releasing its sitter.
    pub fn peek(&self) -> u16 {
        self.number
    }
}

impl<T> AsRef<u16> for Port<T> {
    fn as_ref(&self) -> &u16 {
        &self.number
    }
}

trait Sit {
    type Socket;
    fn sit(port: u16) -> Option<Port<Self::Socket>>;
}

impl Sit for UdpSocket {
    type Socket = UdpSocket;

    fn sit(port: u16) -> Option<Port<Self::Socket>> {
        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        let r = UdpSocket::bind(addr);
        match r {
            Ok(sitter) => Port { number: port, _sitter: sitter }.into(),
            Err(x) if x.kind() == ErrorKind::AddrInUse => None,
            Err(x) => panic!("{}", x),
        }
    }
}

impl Sit for TcpListener {
    type Socket = TcpListener;

    fn sit(port: u16) -> Option<Port<Self::Socket>> {
        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        let r = TcpListener::bind(addr);
        match r {
            Ok(sitter) => Port { number: port, _sitter: sitter }.into(),
            Err(x) if x.kind() == ErrorKind::AddrInUse => None,
            Err(x) => panic!("{}", x),
        }
    }
}

/// Returns a usable private ports (49152-65535) for UDP.
///
/// Example:
///
/// ```rust
/// let port = getrandomport::reserve_udp_port().take();
/// ```
pub fn reserve_udp_port() -> Port<UdpSocket> {
    reserve_port::<UdpSocket, _>(&mut Fastrand)
}

/// Returns a usable private ports (49152-65535) for TCP.
///
/// Example:
///
/// ```rust
/// let port = getrandomport::reserve_tcp_port().take();
/// ```
pub fn reserve_tcp_port() -> Port<TcpListener> {
    reserve_port::<TcpListener, _>(&mut Fastrand)
}

fn reserve_port<T, P>(ports: &mut P) -> Port<T::Socket>
where
    T: Sit,
    P: ProvidePort,
{
    let mut attempts = 0;
    loop {
        if attempts >= 5 {
            panic!("Failed to find usable port after 5 attempts.")
        }
        attempts += 1;
        let port = ports.provide_port();
        match T::sit(port) {
            Some(x) => return x,
            None => (),
        }
    }
}

trait ProvidePort {
    /// Returns number from 49152 to 65535.
    fn provide_port(&mut self) -> u16;
}

/// [Fastrand] backed random port number generator.
///
/// [Fastrand]: https://docs.rs/fastrand/latest/fastrand/
struct Fastrand;

impl ProvidePort for Fastrand {
    fn provide_port(&mut self) -> u16 {
        fastrand::u16(49152..=65535)
    }
}

#[cfg(test)]
mod retry_tests {
    use super::*;

    struct Provider {
        ports: [u16; 2],
        index: usize,
    }

    impl Provider {
        fn new(ports: [u16; 2]) -> Self {
            let index = 0;
            Self { ports, index }
        }
    }

    impl ProvidePort for Provider {
        fn provide_port(&mut self) -> u16 {
            let port = self.ports[self.index];
            self.index += 1;
            port
        }
    }

    #[test]
    fn test_retry_for_udp() {
        let port_1 = reserve_udp_port();

        let port_2 = reserve_udp_port().take();

        let port = reserve_port::<UdpSocket, _>(&mut Provider::new([port_1.peek(), port_2]));
        assert_eq!(port.peek(), port_2)
    }

    #[test]
    fn test_retry_for_tcp() {
        let port_1 = reserve_tcp_port();

        let port_2 = reserve_tcp_port().take();

        let port = reserve_port::<TcpListener, _>(&mut Provider::new([port_1.peek(), port_2]));
        assert_eq!(port.peek(), port_2)
    }
}

#[cfg(test)]
mod max_retries_tests {
    use super::*;
    use std::panic::catch_unwind;

    #[derive(Debug)]
    struct Provider {
        port: u16,
        attempts: u8,
    }

    impl Provider {
        fn new(port: u16) -> Self {
            let attempts = 0;
            Self { port, attempts }
        }
    }

    impl ProvidePort for Provider {
        fn provide_port(&mut self) -> u16 {
            self.attempts += 1;
            if self.attempts > 5 {
                panic!("Expecting attempts to be less than or equal to 5.")
            }
            self.port
        }
    }

    #[test]
    fn test_maximum_retries_for_udp() {
        let port = reserve_udp_port();

        let err = catch_unwind(|| {
            reserve_port::<UdpSocket, _>(&mut Provider::new(port.peek()));
        })
        .unwrap_err();

        let err = err.downcast_ref::<&str>();
        assert!(matches!(err, Some(&x) if x == "Failed to find usable port after 5 attempts."));
    }

    #[test]
    fn test_maximum_retries_for_tcp() {
        let port = reserve_tcp_port();

        let err = catch_unwind(|| {
            reserve_port::<TcpListener, _>(&mut Provider::new(port.peek()));
        })
        .unwrap_err();

        let err = err.downcast_ref::<&str>();
        assert!(matches!(err, Some(&x) if x == "Failed to find usable port after 5 attempts."));
    }
}
