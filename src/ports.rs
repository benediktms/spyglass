use futures::StreamExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;

use crate::common_ports::MOST_COMMON_PORTS_100;
use crate::model::{Port, Subdomain};

use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

pub async fn scan_ports(concurrency: usize, subdomain: Subdomain) -> Subdomain {
    println!("Scanning ports for {}", subdomain.domain);
    let mut ret = subdomain.clone();

    let socket_addresses: Vec<SocketAddr> = format!("{}:1024", subdomain.domain)
        .to_socket_addrs()
        .expect("port scanner: Creating socket address")
        .collect();

    if socket_addresses.is_empty() {
        return subdomain;
    }

    let socket_address = socket_addresses[0];

    let (input_tx, input_rx) = channel(concurrency);
    let (output_tx, output_rx) = channel(concurrency);

    // spawn an independent tokio process to avoid blocking the current thread
    // if the downstream (the main thread) system is slower
    // it can block the current thread (I think?)
    // TODO: research this
    tokio::spawn(async move {
        for port in MOST_COMMON_PORTS_100 {
            let _ = input_tx.send(*port).await;
        }
    });

    let input_rx_stream = ReceiverStream::new(input_rx);
    input_rx_stream
        .for_each_concurrent(concurrency, |port| {
            let output_tx = output_tx.clone();
            async move {
                let port = scan_port(socket_address, port).await;
                if port.is_open {
                    let _ = output_tx.send(port).await;
                }
            }
        })
        .await;

    drop(output_tx);

    let output_rx_stream = ReceiverStream::new(output_rx);
    ret.open_ports = output_rx_stream.collect().await;

    ret
}

async fn scan_port(mut socket_address: SocketAddr, port: u16) -> Port {
    let timeout = Duration::from_secs(3);
    socket_address.set_port(port);

    // let is_open = TcpStream::connect(&socket_address).is_ok();
    let is_open = matches!(
        tokio::time::timeout(timeout, TcpStream::connect(&socket_address)).await,
        Ok(Ok(_)),
    );

    Port { port, is_open }
}
