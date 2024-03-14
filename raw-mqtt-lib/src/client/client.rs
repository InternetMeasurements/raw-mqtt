use bytes::BytesMut;
use log::info;
use mqttbytes::v4::{Connect, ConnectReturnCode, Disconnect, Packet, Publish};
use mqttbytes::QoS;
use std::error::Error;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;

use crate::network::network::Network;
use crate::network::transport::{TcpConfig, Transport};
use crate::{parse_packet, Version, ACK_PACKET_SIZE};

#[derive(Debug, Clone)]
pub struct Client<T> {
    client_id: String,
    pub(crate) pkid: Arc<AtomicU16>,
    host: String,
    server_name: String,
    port: String,
    pub(crate) version: Version,
    pub(crate) network: T,
}

impl<T> Default for Client<T>
where
    T: Network,
{
    fn default() -> Client<T> {
        Client {
            client_id: format!("mqtt-tool-{}", uuid::Uuid::new_v4()),
            pkid: Arc::new(AtomicU16::new(1)),
            host: "localhost".to_string(),
            server_name: "localhost".to_string(),
            port: "1883".to_string(),
            version: Version::V311,
            network: T::new(Transport::TCP(TcpConfig::default())),
        }
    }
}

impl<T> Client<T> {
    pub fn new(
        host: String,
        server_name: String,
        port: String,
        transport: Transport,
        version: Version,
    ) -> Client<T>
    where
        T: Network,
    {
        Client {
            client_id: format!("mqtt-tool-{}", uuid::Uuid::new_v4()),
            host,
            server_name,
            port,
            version,
            network: T::new(transport),
            pkid: Arc::new(AtomicU16::new(1)),
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>>
    where
        T: Network,
    {
        // Establish connection (TCP, TLS, QUIC)
        self.network
            .connect(&self.host, &self.port, &self.server_name)
            .await?;

        // Build connect message
        let send_buffer = match self.version {
            Version::V31 => {
                todo!("MQTT v3.1 not supported yet")
            }
            Version::V311 => {
                let mut send_buffer = BytesMut::new();
                let conn_packet = Connect::new(&self.client_id);
                conn_packet
                    .write(&mut send_buffer)
                    .expect("Packet serialization failed");
                send_buffer
            }
            Version::V5 => {
                todo!("MQTT v5 not supported yet")
            }
        };

        // Send connect message
        self.network.send(send_buffer.as_ref()).await?;

        // Wait for connection ack message
        let mut recv_buffer = self.network.recv(ACK_PACKET_SIZE).await?;

        // Deserialize received message
        let packet = match self.version {
            Version::V31 => {
                todo!("MQTT v3.1 not supported yet")
            }
            Version::V311 => parse_packet(&mut recv_buffer, 1024, &self.version).unwrap(),
            Version::V5 => {
                todo!("MQTT v5 not supported yet")
            }
        };

        info!("Connection ack: {packet:?}");

        // Check received message
        match (self.version, packet) {
            (Version::V31, _) => {
                todo!("MQTT v3.1 not supported yet")
            }
            (Version::V311, Packet::ConnAck(conn_ack)) => {
                if conn_ack.code != ConnectReturnCode::Success {
                    Err(format!("Connection failed: {:?}", conn_ack.code))?
                } else {
                    Ok(())
                }
            }
            (Version::V5, _) => {
                todo!("MQTT v5 not supported yet")
            }
            (_, other) => Err(format!("Unexpected message: {:?}", other))?,
        }
    }

    pub async fn publish(
        &mut self,
        topic: String,
        payload: String,
        qos: QoS,
    ) -> Result<(), Box<dyn Error>>
    where
        T: Network,
    {
        // Build publish message
        let mut pub_req = match self.version {
            Version::V31 => {
                todo!("MQTT v3.1 not supported yet")
            }
            Version::V311 => Publish::new(topic, qos, payload),
            Version::V5 => {
                todo!("MQTT v5 not supported yet")
            }
        };

        // Set packet id
        pub_req.pkid = self.pkid.clone().fetch_add(1, Ordering::SeqCst);
        if pub_req.pkid == 0 {
            pub_req.pkid = self.pkid.clone().fetch_add(1, Ordering::SeqCst);
        }

        // Send publish message
        let mut send_buffer = BytesMut::new();
        pub_req
            .write(&mut send_buffer)
            .expect("Serialization failed");
        self.network.send(send_buffer.as_ref()).await?;

        match qos {
            QoS::AtMostOnce => Ok(()),
            QoS::AtLeastOnce => {
                // Wait for publish ack
                let mut recv_buffer = self.network.recv(4).await?;
                let packet = parse_packet(&mut recv_buffer, 1024, &self.version).unwrap();
                info!("Publish ack: {packet:?}");

                // Check received message
                match packet {
                    Packet::PubAck(pub_ack) => {
                        if pub_ack.pkid != pub_req.pkid {
                            Err(format!(
                                "Publish failed, received different ack < {:} {:} >",
                                pub_req.pkid, pub_ack.pkid
                            ))?
                        } else {
                            Ok(())
                        }
                    }
                    other => Err(format!("Unexpected message: {:?}", other))?,
                }
            }
            QoS::ExactlyOnce => {
                todo!("QoS not supported yet")
            }
        }
    }

    pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>>
    where
        T: Network,
    {
        let mut send_buffer = BytesMut::new();
        match self.version {
            Version::V31 => {
                todo!("MQTT v3.1 not supported yet")
            }
            Version::V311 => {
                Disconnect
                    .write(&mut send_buffer)
                    .expect("Packet serialization failed");
            }
            Version::V5 => {
                todo!("MQTT v5 not supported yet")
            }
        };
        self.network.send(send_buffer.as_ref()).await.unwrap();

        info!("Disconnected from broker");

        Ok(())
    }
}
