#![deny(missing_debug_implementations)]

use std::{
    io::ErrorKind,
    net::{SocketAddr, TcpListener, UdpSocket},
};
use thiserror::Error;

#[derive(Debug)]
pub struct ReservedPort<T> {
    number: u16,
    _res: T,
}

impl<T> ReservedPort<T> {
    /// Takes the port number and release its reservation.
    #[inline]
    pub fn take(self) -> u16 {
        self.number
    }

    /// Returns the port number without releasing its reservation.
    #[inline]
    pub fn peek(&self) -> u16 {
        self.number
    }
}

impl<T> AsRef<u16> for ReservedPort<T> {
    fn as_ref(&self) -> &u16 {
        &self.number
    }
}

pub trait Reservable: private::Sealed {
    type Res;
    fn reserve(port: u16) -> Option<ReservedPort<Self::Res>>;
}

impl Reservable for UdpSocket {
    type Res = UdpSocket;

    fn reserve(port: u16) -> Option<ReservedPort<Self::Res>> {
        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        match UdpSocket::bind(addr) {
            Ok(res) => ReservedPort {
                number: res.local_addr().unwrap().port(),
                _res: res,
            }
            .into(),
            Err(x) if x.kind() == ErrorKind::AddrInUse => None,
            Err(x) => panic!("{}", x),
        }
    }
}

impl Reservable for TcpListener {
    type Res = TcpListener;

    fn reserve(port: u16) -> Option<ReservedPort<Self::Res>> {
        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        match TcpListener::bind(addr) {
            Ok(res) => ReservedPort {
                number: res.local_addr().unwrap().port(),
                _res: res,
            }
            .into(),
            Err(x) if x.kind() == ErrorKind::AddrInUse => None,
            Err(x) => panic!("{}", x),
        }
    }
}

/// Reserves random UDP port from OS.
#[inline]
pub fn reserve_udp_port() -> ReservedPort<UdpSocket> {
    reserve_port::<UdpSocket, _>(Singleton(0)).unwrap()
}

/// Reserves random TCP port from OS.
#[inline]
pub fn reserve_tcp_port() -> ReservedPort<TcpListener> {
    reserve_port::<TcpListener, _>(Singleton(0)).unwrap()
}

pub fn reserve_port<T, P>(mut ports: P) -> Result<ReservedPort<T::Res>, Error>
where
    T: Reservable,
    P: ProvidePorts,
{
    let ports_count = ports.length();
    let mut attempts = 0;
    let port = loop {
        if attempts >= ports_count {
            return Err(Error::Exhausted(attempts));
        }
        let port = ports.get_port();
        match T::reserve(port) {
            Some(x) => break x,
            None => (),
        }
        attempts += 1;
    };
    Ok(port)
}

pub trait ProvidePorts {
    fn get_port(&mut self) -> u16;
    fn length(&self) -> usize;
}

struct Singleton(u16);

impl ProvidePorts for Singleton {
    fn get_port(&mut self) -> u16 {
        self.0
    }

    fn length(&self) -> usize {
        1
    }
}

impl<T> ProvidePorts for T
where
    T: ExactSizeIterator<Item = u16>,
{
    fn get_port(&mut self) -> u16 {
        self.next().unwrap()
    }

    fn length(&self) -> usize {
        self.len()
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to find usable port after {0} attempts")]
    Exhausted(usize),
}

mod private {
    use super::*;

    pub trait Sealed {}
    impl Sealed for UdpSocket {}
    impl Sealed for TcpListener {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_for_udp() {
        let port_1 = reserve_udp_port();
        let port_2 = reserve_udp_port().take();

        let port = reserve_port::<UdpSocket, _>([port_1.peek(), port_2].into_iter()).unwrap();

        assert_eq!(port.peek(), port_2)
    }

    #[test]
    fn test_retry_for_tcp() {
        let port_1 = reserve_tcp_port();
        let port_2 = reserve_tcp_port().take();

        let port = reserve_port::<TcpListener, _>([port_1.peek(), port_2].into_iter()).unwrap();

        assert_eq!(port.peek(), port_2)
    }

    #[test]
    fn test_maximum_retries_for_udp() {
        let port = reserve_udp_port();

        let error = reserve_port::<UdpSocket, _>(Singleton(port.peek())).unwrap_err();

        assert_eq!(
            error.to_string(),
            "failed to find usable port after 1 attempts"
        );
    }

    #[test]
    fn test_maximum_retries_for_tcp() {
        let port = reserve_tcp_port();

        let error = reserve_port::<TcpListener, _>(Singleton(port.peek())).unwrap_err();

        assert_eq!(
            error.to_string(),
            "failed to find usable port after 1 attempts"
        );
    }
}
