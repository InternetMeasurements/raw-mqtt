use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};
use std::str::FromStr;
use std::sync::Arc;

use quinn::{Endpoint, RecvStream, SendStream, TransportConfig};

use tokio::io::{split, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::TlsConnector;

use crate::network::server_verification::{QuinnSkipServerVerification, SkipServerVerification};

#[derive(Debug, Copy, Clone)]
pub enum Transport {
    TCP(TcpConfig),
    TLS(TlsConfig),
    QUIC(QuicConfig),
}

#[derive(Debug, Copy, Clone)]
pub struct TcpConfig {
    pub nagle: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct TlsConfig {
    pub insecure: bool,
    pub nagle: bool,
}
#[derive(Debug, Copy, Clone)]
pub struct QuicConfig {
    pub insecure: bool,
}

impl Default for TcpConfig {
    fn default() -> TcpConfig {
        TcpConfig { nagle: true }
    }
}

impl Default for TlsConfig {
    fn default() -> TlsConfig {
        TlsConfig {
            insecure: false,
            nagle: true,
        }
    }
}

impl Default for QuicConfig {
    fn default() -> QuicConfig {
        QuicConfig { insecure: false }
    }
}

impl FromStr for Transport {
    type Err = String;

    fn from_str(transport: &str) -> Result<Transport, Self::Err> {
        match transport {
            "tcp" => Ok(Transport::TCP(TcpConfig { nagle: true })),
            "tls" => Ok(Transport::TLS(TlsConfig {
                insecure: false,
                nagle: true,
            })),
            "quic" => Ok(Transport::QUIC(QuicConfig { insecure: false })),
            _ => Err("Invalid transport protocol".to_string()),
        }
    }
}

#[derive(Debug)]
pub struct Quic {
    pub(crate) tx_stream: SendStream,
    pub(crate) rx_stream: RecvStream,
}

impl Quic {
    pub async fn new(
        host: &String,
        port: &String,
        insecure: bool,
        sever_name: &String,
    ) -> Result<Quic, Box<dyn Error>> {
        // Set server certificate verification
        let mut tls_config = if insecure {
            // If insecure skip server verification
            quinn_rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(QuinnSkipServerVerification::new())
                .with_no_client_auth()
        } else {
            let mut roots = quinn_rustls::RootCertStore::empty();
            for root in
                rustls_native_certs::load_native_certs().expect("Failed to load native certs")
            {
                roots
                    .add(&quinn_rustls::Certificate(root.to_vec()))
                    .expect("Failed to add root certificate");
            }
            quinn_rustls::ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(roots)
                .with_no_client_auth()
        };

        // Set ALPN field
        let mut alpn: Vec<Vec<u8>> = Vec::new();
        alpn.push("mqtt".as_bytes().to_vec());
        tls_config.alpn_protocols = alpn;

        let mut client_config = quinn::ClientConfig::new(Arc::new(tls_config));

        // Disable unsupported feature segmentation offload
        let mut transport_config = TransportConfig::default();
        transport_config.enable_segmentation_offload(false);
        client_config.transport_config(Arc::new(transport_config));

        let endpoint =
            Endpoint::client("0.0.0.0:0".parse().unwrap()).expect("Failed to build endpoint");

        // Resolve host address
        let socket_addr: Vec<SocketAddr> = format!("{host}:{port}")
            .to_socket_addrs()
            .expect("Failed to resolve host")
            .collect();

        let connection = endpoint
            .connect_with(client_config, socket_addr[0], sever_name.as_str())
            .expect("Connection failed")
            .await?;

        let (tx_stream, rx_stream) = connection.open_bi().await?;

        Ok(Quic {
            tx_stream,
            rx_stream,
        })
    }
}

#[derive(Debug)]
pub struct Tcp {
    pub(crate) rx_stream: ReadHalf<TcpStream>,
    pub(crate) tx_stream: WriteHalf<TcpStream>,
}

impl Tcp {
    pub async fn new(host: &String, port: &String, nagle: bool) -> Result<Tcp, Box<dyn Error>> {
        // Open connection
        let tcp_stream = TcpStream::connect(format!("{host}:{port}")).await?;

        // Enable/Disable Nagle's algorithm
        tcp_stream
            .set_nodelay(nagle == false)
            .expect("Failed to set nodelay");

        // Split into parse_packet and write halves
        let (rx_stream, tx_stream) = split(tcp_stream);

        Ok(Tcp {
            rx_stream,
            tx_stream,
        })
    }
}

#[derive(Debug)]
pub struct Tls {
    pub(crate) rx_stream: ReadHalf<TlsStream<TcpStream>>,
    pub(crate) tx_stream: WriteHalf<TlsStream<TcpStream>>,
}

impl Tls {
    pub async fn new(
        host: &String,
        port: &String,
        nagle: bool,
        insecure: bool,
        server_name: &String,
    ) -> Result<Tls, Box<dyn Error>> {
        let tcp_stream = TcpStream::connect(format!("{host}:{port}")).await?;

        // Enable/Disable Nagle's algorithm
        tcp_stream
            .set_nodelay(nagle == false)
            .expect("Failed to set nodelay");

        // Set server certificate verification
        let tls_client_config = if insecure {
            // If insecure skip server verification
            tokio_rustls::rustls::ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(SkipServerVerification::new())
                .with_no_client_auth()
        } else {
            let mut roots = tokio_rustls::rustls::RootCertStore::empty();
            roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            tokio_rustls::rustls::ClientConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth()
        };

        let connector = TlsConnector::from(Arc::new(tls_client_config));
        let tls_stream = connector
            .connect(
                ServerName::try_from(server_name.as_str())?.to_owned(),
                tcp_stream,
            )
            .await?;

        // Split into parse_packet and write halves
        let (rx_stream, tx_stream) = split(tls_stream);

        Ok(Tls {
            rx_stream,
            tx_stream,
        })
    }
}
