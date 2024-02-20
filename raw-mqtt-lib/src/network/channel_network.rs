use crate::network::channel::{
    BoundedReceiver, BoundedSender, ChannelReceiver, ChannelResult, ChannelSender, SingleLifoQueue,
    UnboundedReceiver, UnboundedSender,
};
use async_trait::async_trait;
use bytes::BytesMut;
use std::error::Error;
use std::fmt::Debug;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::network::network::Network;
use crate::network::transport::{Quic, Tcp, Tls, Transport};

const DEFAULT_QUEUE: i64 = 1024;

#[derive(Debug, Clone)]
pub struct ChannelNetwork {
    tracker: TaskTracker,
    cancellation_token: CancellationToken,
    transport: Transport,
    insecure: bool,
    queue: i64,
    to_sender: Option<ChannelSender>,
    from_receiver: Option<async_channel::Receiver<BytesMut>>,
}

fn spawn_sender(
    tracker: TaskTracker,
    token: CancellationToken,
    mut from_producer: ChannelReceiver,
    mut tx_stream: impl AsyncWriteExt + Unpin + Send + 'static,
) {
    tracker.spawn(async move {
        'main: loop {
            tokio::select! {
                _ = token.cancelled() => {
                    break 'main;
                },
                Ok(buf) = from_producer.receive() => {
                    tx_stream.write_all(&buf).await.unwrap();
                }
            }
        }
    });
}

fn spawn_receiver(
    tracker: TaskTracker,
    token: CancellationToken,
    to_consumer: async_channel::Sender<BytesMut>,
    mut rx_stream: impl AsyncReadExt + Unpin + Send + 'static,
) {
    tracker.spawn(async move {
        let mut buffer = vec![0_u8; 4];
        'main: loop {
            tokio::select! {
                _ = token.cancelled() => {
                    break 'main;
                },
                _ = rx_stream.read_exact(&mut buffer) => {
                    to_consumer.send(BytesMut::from(buffer.as_slice())).await.unwrap();
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
            queue: DEFAULT_QUEUE,
            to_sender: None,
            from_receiver: None,
        }
    }

    async fn connect(
        &mut self,
        host: &String,
        port: &String,
        server_name: &String,
    ) -> Result<(), Box<dyn Error>> {
        let from_producer: ChannelReceiver;
        let to_sender: ChannelSender;

        let to_consumer;
        let from_receiver;

        if self.queue == -1 {
            // Unbounded FIFO queue
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<BytesMut>();
            from_producer = ChannelReceiver::Unbounded(UnboundedReceiver::new(rx));
            to_sender = ChannelSender::Unbounded(UnboundedSender::new(tx));
            (to_consumer, from_receiver) = async_channel::unbounded();
        } else if self.queue == 0 {
            // LIFO queue
            let lifo = SingleLifoQueue::new();
            from_producer = ChannelReceiver::Lifo(lifo.clone());
            to_sender = ChannelSender::Lifo(lifo.clone());
            (to_consumer, from_receiver) = async_channel::unbounded();
        } else {
            // Bounded FIFO queue
            let (tx, rx) = tokio::sync::mpsc::channel::<BytesMut>(self.queue as usize);
            from_producer = ChannelReceiver::Bounded(BoundedReceiver::new(rx));
            to_sender = ChannelSender::Bounded(BoundedSender::new(tx));
            (to_consumer, from_receiver) = async_channel::unbounded();
        }

        match self.transport {
            Transport::TCP => {
                let tcp = Tcp::new(host, port).await?;

                // Sender task
                spawn_sender(
                    self.tracker.clone(),
                    self.cancellation_token.clone(),
                    from_producer,
                    tcp.tx_stream,
                );

                // Receiver task
                spawn_receiver(
                    self.tracker.clone(),
                    self.cancellation_token.clone(),
                    to_consumer,
                    tcp.rx_stream,
                );
            }
            Transport::TLS => {
                let tls = Tls::new(host, port, &self.insecure, server_name).await?;

                // Sender task
                spawn_sender(
                    self.tracker.clone(),
                    self.cancellation_token.clone(),
                    from_producer,
                    tls.tx_stream,
                );

                // Receiver task
                spawn_receiver(
                    self.tracker.clone(),
                    self.cancellation_token.clone(),
                    to_consumer,
                    tls.rx_stream,
                );
            }
            Transport::QUIC => {
                let quic = Quic::new(host, port, &self.insecure, &server_name).await?;

                // Sender task
                spawn_sender(
                    self.tracker.clone(),
                    self.cancellation_token.clone(),
                    from_producer,
                    quic.tx_stream,
                );

                // Receiver task
                spawn_receiver(
                    self.tracker.clone(),
                    self.cancellation_token.clone(),
                    to_consumer,
                    quic.rx_stream,
                );
            }
        };

        self.from_receiver = Some(from_receiver);
        self.to_sender = Some(to_sender);

        Ok(())
    }

    async fn send(&mut self, tx_buffer: &[u8]) -> Result<(), Box<dyn Error>> {
        match self.to_sender {
            Some(ref mut tx_stream) => {
                let res = tx_stream.send(BytesMut::from(tx_buffer)).await;
                match res {
                    Ok(ChannelResult::Added) => Ok(()),
                    Ok(ChannelResult::Replaced) => Err("Replaced into queue")?,
                    Err(_) => Err("Failed to insert in queue")?,
                }
            }
            None => Err("No send stream available")?,
        }
    }

    async fn recv(&mut self, size: usize) -> Result<BytesMut, Box<dyn Error>> {
        let mut rx_buffer = vec![0_u8; size];

        match self.from_receiver {
            Some(ref mut rx_stream) => {
                rx_buffer = rx_stream.recv().await.unwrap().to_vec();
            }
            None => Err("No send stream available")?,
        }

        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(rx_buffer.as_slice());

        Ok(buffer)
    }
}

impl ChannelNetwork {
    pub async fn close(&self) -> Result<(), Box<dyn Error>> {
        self.cancellation_token.cancel();
        self.tracker.close();
        self.tracker.wait().await;
        Ok(())
    }

    pub fn set_queue(&mut self, queue: i64) {
        self.queue = queue;
    }
}
