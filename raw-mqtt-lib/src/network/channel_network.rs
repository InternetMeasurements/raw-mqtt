use std::error::Error;
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::Sender;
use async_channel::Receiver;
use async_trait::async_trait;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::network::network::Network;
use crate::network::transport::{Quic, Tcp, Tls, Transport};

#[derive(Debug, Clone)]
pub struct ChannelNetwork {
    tracker: TaskTracker,
    cancellation_token: CancellationToken,
    transport: Transport,
    insecure: bool,
    tx_stream: Option<Sender<BytesMut>>,
    rx_stream: Option<Receiver<BytesMut>>
}

fn spawn_sender(tracker: TaskTracker, token: CancellationToken, mut rx_sender: tokio::sync::mpsc::Receiver<BytesMut>, mut tx_stream: impl AsyncWriteExt + Unpin + Send + 'static) {
    tracker.spawn(async move {
        'main: loop {
            tokio::select! {
                _ = token.cancelled() => {
                    break 'main;
                },
                Some(buf) = rx_sender.recv() => {
                    tx_stream.write_all(&buf).await.unwrap();
                }
            }
        }
    });
}

fn spawn_receiver(tracker: TaskTracker, token: CancellationToken, tx_receiver: async_channel::Sender<BytesMut>, mut rx_stream: impl AsyncReadExt + Unpin + Send + 'static) {
    tracker.spawn(async move {
        let mut buffer = vec![0_u8; 4];
        'main: loop {
            tokio::select! {
                _ = token.cancelled() => {
                    break 'main;
                },
                _ = rx_stream.read_exact(&mut buffer) => {
                    tx_receiver.send(BytesMut::from(buffer.as_slice())).await.unwrap();
                }
            }
        }
    });
}

#[async_trait]
impl Network for ChannelNetwork {
    fn new(transport: Transport, insecure: bool) -> ChannelNetwork {
        let tracker = TaskTracker::new();
        let cancellation_token = CancellationToken::new();

        ChannelNetwork {
            tracker,
            cancellation_token,
            transport,
            insecure,
            tx_stream: None,
            rx_stream: None
        }
    }

    async fn connect(&mut self, host: &String, port: &String, server_name: &String) -> Result<(), Box<dyn Error>> {
        let (tx_sender, rx_sender) = tokio::sync::mpsc::channel::<BytesMut>(1024);
        let (tx_receiver, rx_receiver) = async_channel::bounded(1024);

        match self.transport {
            Transport::TCP => {
                let tcp = Tcp::new(host, port).await?;

                // Sender task
                spawn_sender(self.tracker.clone(), self.cancellation_token.clone(), rx_sender, tcp.tx_stream);

                // Receiver task
                spawn_receiver(self.tracker.clone(), self.cancellation_token.clone(), tx_receiver, tcp.rx_stream);
            },
            Transport::TLS => {
                let tls = Tls::new(host, port, &self.insecure, server_name).await?;

                // Sender task
                spawn_sender(self.tracker.clone(), self.cancellation_token.clone(), rx_sender, tls.tx_stream);

                // Receiver task
                spawn_receiver(self.tracker.clone(), self.cancellation_token.clone(), tx_receiver, tls.rx_stream);
            },
            Transport::QUIC => {
                let quic = Quic::new(host, port, &self.insecure).await?;

                // Sender task
                spawn_sender(self.tracker.clone(), self.cancellation_token.clone(), rx_sender, quic.tx_stream);

                // Receiver task
                spawn_receiver(self.tracker.clone(), self.cancellation_token.clone(), tx_receiver, quic.rx_stream);
            }
        };

        self.rx_stream = Some(rx_receiver);
        self.tx_stream = Some(tx_sender);

        Ok(())
    }

    async fn send(&mut self, tx_buffer: &[u8]) -> Result<(), Box<dyn Error>>{
        match self.tx_stream {
            Some(ref mut tx_stream) => {
                tx_stream.send(BytesMut::from(tx_buffer)).await?
            },
            None => {
                Err("No send stream available")?
            }
        }

        Ok(())
    }

    async fn recv(&mut self, size: usize) -> Result<BytesMut, Box<dyn Error>>{
        let mut rx_buffer = vec![0_u8; size];

        match self.rx_stream {
            Some(ref mut rx_stream) => {
                rx_buffer = rx_stream.recv().await.unwrap().to_vec();
            },
            None => {
                Err("No send stream available")?
            }
        }

        let mut buffer= BytesMut::new();
        buffer.extend_from_slice(rx_buffer.as_slice());

        Ok(buffer)
    }

}

impl ChannelNetwork {
    pub async fn close(&self) -> Result<(), Box<dyn Error>>{
        self.cancellation_token.cancel();
        self.tracker.close();
        self.tracker.wait().await;
        Ok(())
    }
}