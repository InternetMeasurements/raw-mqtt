use clap::Parser;
use log::{debug, info, LevelFilter};
use mqttbytes::QoS;
use raw_mqtt::client::stream_client::StreamMqttClient;
use raw_mqtt::network::transport::Transport;
use raw_mqtt::utility::argument_parser::Request;
use raw_mqtt::utility::stream_argument_parser::MqttStreamCli;
use raw_mqtt::Version;
use std::error;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn error::Error>> {
    // Parse command line arguments
    let (request, args, message_payload, rate, duration, queue, nagle) =
        match MqttStreamCli::parse() {
            MqttStreamCli::Publish(stream_args) => {
                let payload = match stream_args.publish_args.size {
                    Some(size) => String::from_utf8(vec![127_u8; size]).unwrap(),
                    None => stream_args.publish_args.message.unwrap(),
                };
                (
                    Request::Publish,
                    stream_args.publish_args.common_args,
                    Some(payload),
                    if stream_args.rate > 0.0 {
                        Some(stream_args.rate)
                    } else {
                        None
                    },
                    if stream_args.duration > 0 {
                        Some(stream_args.duration)
                    } else {
                        None
                    },
                    Some(stream_args.queue),
                    Some(!stream_args.nagle_off),
                )
            }
            MqttStreamCli::Subscribe(subscribe_args) => (
                Request::Subscribe,
                subscribe_args.common_args,
                None,
                None,
                None,
                None,
                None,
            ),
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
    let mut transport = Transport::from_str(args.transport.as_str()).unwrap();
    match transport {
        Transport::TCP(ref mut config) => {
            config.nagle = nagle.unwrap();
        }
        Transport::TLS(ref mut config) => {
            config.insecure = args.insecure;
            config.nagle = nagle.unwrap();
        }
        Transport::QUIC(ref mut config) => {
            config.insecure = args.insecure;
        }
    }

    let qos = match { args.qos } {
        0 => QoS::AtMostOnce,
        1 => QoS::AtLeastOnce,
        2 => QoS::ExactlyOnce,
        _ => panic!("Invalid QoS value"),
    };

    let mut client = StreamMqttClient::new(
        args.host,
        args.server_name,
        args.port.to_string(),
        transport,
        proto_version,
    );

    // Set queue size
    if queue.is_some() {
        client.set_queue(queue.unwrap());
    }

    client.connect().await?;
    match request {
        Request::Publish => {
            let payload = message_payload.unwrap();
            match rate {
                Some(rate) => {
                    let num_messages = duration.unwrap() * rate as usize;
                    let rate = 1.0 / rate;

                    info!("Sending {} messages", num_messages);

                    let mut interval_timer = tokio::time::interval(
                        chrono::Duration::microseconds(
                            Duration::from_secs_f64(rate).as_micros() as i64
                        )
                        .to_std()
                        .unwrap(),
                    );

                    for _ in 0..num_messages {
                        // Wait for the next tick
                        interval_timer.tick().await;

                        // Generate new data
                        let generation_timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Time went backwards")
                            .as_nanos();

                        let topic = args.topic.to_string();

                        // Publish new message (stream publish)
                        client
                            .stream_publish(
                                topic.to_string(),
                                format!("{:}{:}", generation_timestamp, payload)[0..payload.len()]
                                    .to_string(),
                                qos,
                            )
                            .await?;
                    }
                }
                None => {
                    info!("Publishing message of size: {}", payload.len());
                    client.publish(args.topic, payload, qos).await?;
                }
            }
        }
        Request::Subscribe => {
            todo!("Subscribe not implemented yet")
        }
    }

    client.disconnect().await?;

    Ok(())
}
