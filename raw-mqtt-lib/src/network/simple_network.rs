#![allow(dead_code)]

use std::error::Error;
use async_trait::async_trait;
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::network::network::Network;
use crate::network::transport::{Quic, Tcp, Tls, Transport};

#[derive(Debug)]
pub enum SimpleNetwork {
    TCP(Option<Tcp>, bool),
    TLS(Option<Tls>, bool),
    QUIC(Option<Quic>, bool)
}
#[async_trait]
impl Network for SimpleNetwork {
    fn new(transport: Transport, insecure: bool) -> SimpleNetwork {
        match transport {
            Transport::TCP => {
                SimpleNetwork::TCP(None, insecure)
            },
            Transport::TLS => {
                SimpleNetwork::TLS(None, insecure)
            },
            Transport::QUIC => {
                SimpleNetwork::QUIC(None, insecure)
            }
        }
    }

    async fn connect(&mut self, host: &String, port: &String, server_name: &String) -> Result<(), Box<dyn Error>> {
        match self {
            SimpleNetwork::TCP(tcp, _) => {
                *tcp = Some(Tcp::new(host, port).await?);
            },
            SimpleNetwork::TLS(tls, insecure) => {
                *tls = Some(Tls::new(host, port, insecure, &server_name).await?);
            },
            SimpleNetwork::QUIC(quic, insecure) => {
                *quic = Some(Quic::new(host, port, insecure, &server_name).await?);
            }
        }
        Ok(())
    }

    async fn send(&mut self, tx_buffer: &[u8]) -> Result<(), Box<dyn Error>>{
        match self {
            SimpleNetwork::TCP(Some(tcp),_) => {
                tcp.tx_stream.write_all(tx_buffer).await?
            },
            SimpleNetwork::TLS(Some(tls),_) => {
                tls.tx_stream.write_all(tx_buffer).await?

            },
            SimpleNetwork::QUIC(Some(quic),_) => {
                quic.tx_stream.write_all(tx_buffer).await?
            }
            _ => { Err("No send stream available")? }
        }

        Ok(())
    }

    async fn recv(&mut self, size: usize) -> Result<BytesMut, Box<dyn Error>>{
        let mut rx_buffer = vec![0_u8; size];

        match self {
            SimpleNetwork::TCP(Some(tcp),_) => {
                tcp.rx_stream.read_exact(&mut rx_buffer).await?;
            }
            SimpleNetwork::TLS(Some(tls),_) => {
                tls.rx_stream.read_exact(&mut rx_buffer).await?;
            },
            SimpleNetwork::QUIC(Some(quic),_) => {
                quic.rx_stream.read_exact(&mut rx_buffer).await?;
            }
            _ => { Err("No send stream available")? }
        }

        let mut buffer= BytesMut::new();
        buffer.extend_from_slice(rx_buffer.as_slice());

        Ok(buffer)
    }
}
