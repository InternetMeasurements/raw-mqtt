# Raw-MQTT

Simple Rust MQTT client library and tools supporting TCP, TLS and QUIC transport layers. 

---

## Structure
The repository is structured as follows:
- [raw-mqtt-lib](raw-mqtt-lib) - The main library crate
- [raw-mqtt-cli](raw-mqtt-cli) - A simple command line client tool
- [raw-mqtt-stream-cli](raw-mqtt-stream-cli) - A simple command line client tool for streaming requests

---

## Dependencies
|                        |     Crate      |      Version       |                                                  Documentation                                                   |        Notes         |
|:----------------------:|:--------------:|:------------------:|:----------------------------------------------------------------------------------------------------------------:|:--------------------:|
|  Asynchronous Runtime  |     tokio      |       1.35.1       |        [<img src="docs/rust.svg" width="32" height="32"/>](https://docs.rs/tokio/1.35.1/tokio/index.html)        |          -           |
|          Quic          |     quinn      |       0.10.2       |        [<img src="docs/rust.svg" width="32" height="32"/>](https://docs.rs/quinn/0.10.2/quinn/index.html)        |   Based on rustls    |
|          Tls           |  tokio-rustls  |       0.25.0       | [<img src="docs/rust.svg" width="32" height="32"/>](https://docs.rs/tokio-rustls/0.25.0/tokio_rustls/index.html) |   Based on rustls    |

---

## TODO
 - Subscribe operation
 - Client authentication
 - Protocol version 3.1 and 5
 - QoS 2
 - Keep alive mechanism
 - Pretty error messages
---

## Known Issues
- Quic transport protocol is not compatible with EMQX broker
---

