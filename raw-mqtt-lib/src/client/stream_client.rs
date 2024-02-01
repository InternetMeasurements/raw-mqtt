use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicU16, Ordering};
use mqttbytes::v4::Publish;
use mqttbytes::QoS;
use bytes::{BytesMut};
use log::debug;
use tokio::sync::mpsc::Sender;
use tokio::task::yield_now;

use crate::network::channel_network::ChannelNetwork;
use crate::network::transport::Transport;
use crate::{ACK_PACKET_SIZE, MqttMessage, parse_packet, Version};
use crate::client::client::Client;
use crate::network::network::Network;


#[derive(Debug)]
pub struct StreamMqttClient {
    _client: Client<ChannelNetwork>,
    pending_acks: Arc<AtomicU16>,
    tx: Option<Sender<MqttMessage>>
}

impl Default for StreamMqttClient {
    fn default() -> StreamMqttClient {
        StreamMqttClient{
            _client: Client::default(),
            pending_acks: Arc::new(AtomicU16::new(0)),
            tx: None
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
            pending_acks: Arc::new(AtomicU16::new(0)),
            tx: None
        }
    }

    /**
     * Connect to broker.
     */
    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        self._client.connect().await?;

        let (tx, mut rx) = tokio::sync::mpsc::channel::<MqttMessage>(1024);
        self.tx = Some(tx);
        let network = self._client.network.clone();
        let version = self._client.version.clone();
        let pkid = self._client.pkid.clone();
        let pending_acks = self.pending_acks.clone();


        let _ = tokio::spawn(async move {

            loop {
                let v = version.clone();
                let message = rx.recv().await.unwrap();

                let mut pub_req = Publish::new(message.topic, message.qos, message.payload);
                let mut send_buffer = BytesMut::new();
                let pending_acks_tx = pending_acks.clone();
                let pending_acks_rx = pending_acks.clone();

                pub_req.pkid = pkid.clone().fetch_add(1, Ordering::SeqCst);
                if pub_req.pkid == 0 {
                    pub_req.pkid = pkid.clone().fetch_add(1, Ordering::SeqCst);
                }
                pub_req.write(&mut send_buffer).expect("Packet serialization failed");

                let mut send_network = network.clone();
                let mut recv_network = network.clone();

                tokio::spawn(async move {
                    send_network.send(send_buffer.as_ref()).await.unwrap();
                    if message.qos == QoS::AtMostOnce {
                        pending_acks_tx.clone().fetch_sub(1, Ordering::SeqCst);
                    }

                });

                if message.qos == QoS::AtLeastOnce {
                    tokio::spawn(async move {
                        let mut recv_buffer = recv_network.recv(ACK_PACKET_SIZE).await.unwrap();
                        let packet = parse_packet(&mut recv_buffer, 1024, &v).unwrap();
                        debug!("Received ack: {:?}", packet);
                        pending_acks_rx.clone().fetch_sub(1, Ordering::SeqCst);
                    });
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
        while self.pending_acks.fetch_add(0, Ordering::SeqCst) > 0 {
            yield_now().await;
        }
        debug!("{:?}", self.pending_acks);
        self._client.disconnect().await?;
        self._client.network.close().await
    }

    /**
     * Publish a stream of messages to a topic.
     */
    pub async fn stream_publish(&self, topic: String, payload: String, qos: QoS) -> Result<(), Box<dyn Error>> {
        self.tx.as_ref().unwrap().send(MqttMessage{
            topic,
            payload,
            qos
        }).await?;

        self.pending_acks.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

}
