use clap::Parser;
use log::{debug, info, LevelFilter};
use mqttbytes::QoS;
use std::error;
use std::str::FromStr;

use raw_mqtt::client::simple_client::SimpleMqttClient;
use raw_mqtt::network::transport::Transport;
use raw_mqtt::utility::argument_parser::{MqttCli, Request};
use raw_mqtt::Version;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn error::Error>> {
    // Parse command line arguments
    let (request, args, message_payload) = match MqttCli::parse() {
        MqttCli::Publish(args) => {
            let payload = match args.size {
                Some(size) => String::from_utf8(vec![127_u8; size]).unwrap(),
                None => args.message.unwrap(),
            };
            (Request::Publish, args.args, Some(payload))
        }
        MqttCli::Subscribe(args) => (Request::Subscribe, args.args, None),
    };

    // Set log level
    env_logger::builder()
        .filter_level(if args.debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        })
        .init();

    debug!("{:?}", args);

    let proto_version = Version::from_str(args.proto_version.as_str()).unwrap();
    let transport = Transport::from_str(args.transport.as_str()).unwrap();
    let qos = match { args.qos } {
        0 => QoS::AtMostOnce,
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => panic!("Invalid QoS value"),
    };

    let mut client = SimpleMqttClient::new(
        args.host,
        args.server_name,
        args.port.to_string(),
        transport,
        proto_version,
        args.insecure,
    );

    client.connect().await?;

    match request {
        Request::Publish => {
            let message_payload = message_payload.unwrap();
            info!("Publishing message of size: {}", message_payload.len());
            client.publish(args.topic, message_payload, qos).await?;
        }
        Request::Subscribe => {
            todo!("Subscribe not implemented")
        }
    }

    client.disconnect().await?;

    Ok(())
}
