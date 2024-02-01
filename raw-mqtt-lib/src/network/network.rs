use std::error::Error;
use async_trait::async_trait;
use bytes::BytesMut;
use crate::network::transport::Transport;

#[async_trait]
pub trait Network {
    fn new(transport: Transport, insecure: bool) -> Self;
    async fn connect(&mut self, host: &String, port: &String, server_name: &String) -> Result<(), Box<dyn Error>>;
    async fn send(&mut self, tx_buffer: &[u8]) -> Result<(), Box<dyn Error>>;
    async fn recv(&mut self, size: usize) -> Result<BytesMut, Box<dyn Error>>;
}