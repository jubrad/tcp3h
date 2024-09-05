use std::net::SocketAddr;
use std::str::FromStr;

use clap::{Arg, Command};
use proxy_header::{ParseConfig, ProxiedAddress, ProxyHeader};
use tokio::io::AsyncWriteExt;

use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup CLI
    let matches = Command::new("tcp3h")
        .version("0.1")
        .author("jubrad")
        .about("A TCP proxy that forwards client IP using Proxy Protocol v2")
        .arg(
            Arg::new("listen")
                .short('l')
                .long("listen")
                .value_name("LISTEN_ADDR")
                .help("The address to listen on")
                .required(true)
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            Arg::new("backend")
                .short('b')
                .long("backend")
                .value_name("BACKEND_ADDR")
                .help("The backend server to forward to")
                .required(true)
                .value_parser(clap::value_parser!(String)),
        )
        .get_matches();

    let listen_addr: &String = matches.get_one("listen").unwrap();
    let backend_addr: &String = matches.get_one("backend").unwrap();

    let listen_addr = SocketAddr::from_str(listen_addr).unwrap();
    let backend_addr = SocketAddr::from_str(backend_addr).unwrap();

    // Start listening on the given address
    let listener = TcpListener::bind(listen_addr).await?;
    println!("Listening on {}", listen_addr);

    loop {
        let (mut client_socket, _client_addr) = listener.accept().await?;

        tokio::spawn(async move {
            match TcpStream::connect(backend_addr).await {
                Ok(mut backend_socket) => {
                    // Send Proxy Protocol v2 header
                    let client_addr = client_socket.peer_addr().unwrap();
                    let addrs = ProxiedAddress::stream(client_addr, backend_addr);
                    let header = ProxyHeader::with_address(addrs);

                    let mut buf = [0u8; 1024];
                    match header.encode_to_slice_v2(&mut buf) {
                        Ok(len) => {
                            // We're parsing the header directly out of the
                            // buffer not efficient, but I want to ensure it's
                            // what the downstream would parse if using the same
                            // library.
                            match ProxyHeader::parse(&buf[..len], ParseConfig::default()) {
                                Ok(decoded_header) => {
                                    println!(
                                        "Decoded Proxy Protocol v2 Header: {:?}",
                                        decoded_header
                                    );
                                }
                                Err(e) => {
                                    eprintln!("Failed to decode Proxy Protocol header: {}", e);
                                    return;
                                }
                            }

                            if let Err(e) = backend_socket.write_all(&buf[..len]).await {
                                eprintln!("Failed to write Proxy Protocol header: {}", e);
                                return;
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to encode Proxy Protocol header: {}", e);
                            return;
                        }
                    }

                    // Relay data between client and backend
                    let (mut cr, mut cw) = client_socket.split();
                    let (mut br, mut bw) = backend_socket.split();

                    let client_to_backend = tokio::io::copy(&mut cr, &mut bw);
                    let backend_to_client = tokio::io::copy(&mut br, &mut cw);

                    if let Err(e) = tokio::try_join!(client_to_backend, backend_to_client) {
                        eprintln!("Failed to relay data: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to backend {}: {}", backend_addr, e);
                }
            }
        });
    }
}
