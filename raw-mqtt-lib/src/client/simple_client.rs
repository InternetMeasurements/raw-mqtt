use mqttbytes::QoS;
use std::error::Error;

use crate::client::client::Client;
use crate::network::simple_network::SimpleNetwork;
use crate::network::transport::Transport;
use crate::Version;

#[derive(Debug)]
pub struct SimpleMqttClient {
    _client: Client<SimpleNetwork>,
}

impl Default for SimpleMqttClient {
    fn default() -> SimpleMqttClient {
        SimpleMqttClient {
            _client: Client::default(),
        }
    }
}

impl SimpleMqttClient {
    pub fn new(
        host: String,
        server_name: String,
        port: String,
        transport: Transport,
        version: Version,
    ) -> SimpleMqttClient {
        SimpleMqttClient {
            _client: Client::new(host, server_name, port, transport, version),
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        self._client.connect().await
    }

    pub async fn publish(
        &mut self,
        topic: String,
        payload: String,
        qos: QoS,
    ) -> Result<(), Box<dyn Error>> {
        self._client.publish(topic, payload, qos).await
    }

    pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        self._client.disconnect().await
    }
}
