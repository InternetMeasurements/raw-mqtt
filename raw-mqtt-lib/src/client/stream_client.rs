use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use mqttbytes::v4::Publish;
use mqttbytes::QoS;
use bytes::{BytesMut};
use log::debug;
use tokio::task::yield_now;
use tokio_util::sync::CancellationToken;

use crate::network::channel_network::ChannelNetwork;
use crate::network::transport::Transport;
use crate::{ACK_PACKET_SIZE, parse_packet, Version};
use crate::client::client::Client;
use crate::network::network::Network;


#[derive(Debug, Clone)]
pub struct StreamMqttClient {
    _client: Client<ChannelNetwork>,
    pending_requests: Arc<AtomicU16>,
    cancellation_tkn: CancellationToken
}

impl Default for StreamMqttClient {
    fn default() -> StreamMqttClient {
        StreamMqttClient{
            _client: Client::default(),
            pending_requests: Arc::new(AtomicU16::new(0)),
            cancellation_tkn: CancellationToken::new()
        }
    }
}

impl StreamMqttClient {

    pub fn new(host: String, server_name: String, port: String, transport: Transport, version: Version, insecure: bool) -> StreamMqttClient {

        StreamMqttClient {
            _client: Client::new(
                host,
                server_name,
                port,
                transport,
                version,
                insecure
            ),
            pending_requests: Arc::new(AtomicU16::new(0)),
            cancellation_tkn: CancellationToken::new()
        }
    }

    /**
     * Connect to broker.
     */
    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        self._client.connect().await?;

        let pending_requests = self.pending_requests.clone();
        let mut recv_network = self._client.network.clone();
        let version = self._client.version;
        let cancellation_tkn = self.cancellation_tkn.clone();

        // Spawn receiver task (for acks)
        tokio::spawn(async move {
            'rx_loop: loop {
                tokio::select! {
                _ = cancellation_tkn.cancelled() => {
                    break 'rx_loop;
                },
                recv_buffer = recv_network.recv(ACK_PACKET_SIZE) => {
                    let packet = parse_packet(&mut recv_buffer.unwrap(), 1024, &version).unwrap();
                    debug!("Received ack: {:?}", packet);
                    pending_requests.fetch_sub(1, Ordering::SeqCst);
                }
            }
            }
        });

        Ok(())
    }

    /**
     * Publish a single message to a topic.
     */
    pub async fn publish(&mut self, topic: String, payload: String, qos: QoS) -> Result<(), Box<dyn Error>> {
        self._client.publish(topic, payload, qos).await
    }

    /**
     * Disconnect from broker and stop network tasks. The client cannot be reused after this call.
     */
    pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {

        while self.pending_requests.load(Ordering::SeqCst) > 0 {
            yield_now().await;
        }
        debug!("{:?}", self.pending_requests);
        self._client.disconnect().await?;
        self.cancellation_tkn.cancel();
        self._client.network.close().await
    }

    /**
     * Publish a stream of messages to a topic.
     */
    pub async fn stream_publish(&self, topic: String, payload: String, qos: QoS) -> Result<(), Box<dyn Error>> {
        let mut network = self._client.network.clone();
        let mut pub_req = Publish::new(topic, qos, payload);

        // Set packet id (if needed)
        if qos == QoS::AtLeastOnce || qos == QoS::ExactlyOnce {
            while pub_req.pkid == 0 {
                pub_req.pkid = self._client.pkid.fetch_add(1, Ordering::SeqCst);
            }
        }

        // Serialize packet
        let mut send_buffer = BytesMut::new();
        pub_req.write(&mut send_buffer).expect("Packet serialization failed");
        
        let res = network.send(send_buffer.as_ref()).await;
        match res {
            Ok(_) => {
                if qos != QoS::AtMostOnce {
                    self.pending_requests.fetch_add(1, Ordering::SeqCst);
                }
            },
            _ => () // Do nothing, the message has been dropped (LIFO queue)
        }
        Ok(())
    }
    

    /**
     * Set the queue size for the network.
     */
    pub fn set_queue(&mut self, queue: i64){
        self._client.network.set_queue(queue);
    }
}
