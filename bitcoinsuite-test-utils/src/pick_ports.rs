use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, TcpListener, ToSocketAddrs};

use rand::Rng;

use crate::{error::Result, UtilError};

/// Try to bind to a socket using TCP
fn test_bind_tcp(addr: impl ToSocketAddrs) -> Option<u16> {
    Some(TcpListener::bind(addr).ok()?.local_addr().ok()?.port())
}

/// Check if a port is free on TCP
pub fn is_free_tcp(port: u16) -> bool {
    let ipv4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    let ipv6 = SocketAddrV6::new(Ipv6Addr::LOCALHOST, port, 0, 0);

    test_bind_tcp(ipv6).is_some() && test_bind_tcp(ipv4).is_some()
}

/// Asks the OS for a free port
fn ask_free_tcp_port() -> Option<u16> {
    let ipv4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    let ipv6 = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 0, 0, 0);

    test_bind_tcp(ipv6).or_else(|| test_bind_tcp(ipv4))
}

fn has_item<T: Eq>(items: &[T], item_test: &T) -> bool {
    items.iter().any(|item| item == item_test)
}

pub fn pick_ports(num_ports: usize) -> Result<Vec<u16>> {
    let mut ports = Vec::with_capacity(num_ports);
    'port_loop: for _ in 0..num_ports {
        let mut rng = rand::thread_rng();

        // Try random port first
        for _ in 0..40 {
            let port = rng.gen_range(15000..25000);
            if has_item(&ports, &port) {
                continue;
            }
            if is_free_tcp(port) {
                ports.push(port);
                continue 'port_loop;
            }
        }

        // Ask the OS for a port
        for _ in 0..40 {
            if let Some(port) = ask_free_tcp_port() {
                if has_item(&ports, &port) {
                    continue;
                }
                ports.push(port);
                continue 'port_loop;
            }
        }

        return Err(UtilError::PickPortsFailed);
    }

    Ok(ports)
}
