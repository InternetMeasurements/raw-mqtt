use std::str::FromStr;
use mqttbytes::v4::{Packet, Connect, ConnAck, PubAck, PubComp, Publish, PubRec,
                    PubRel, SubAck, Subscribe, UnsubAck, Unsubscribe};
use mqttbytes::{check, PacketType, QoS};
use bytes::{BytesMut};

pub mod network;
pub mod client;
pub mod utility;

pub const ACK_PACKET_SIZE: usize = 4;

pub(crate) fn parse_packet(stream: &mut BytesMut, max_size: usize, version: &Version) -> Result<Packet, mqttbytes::Error> {
    match version { 
        Version::V31 => { todo!("V3.1 not supported yet"); }
        Version::V311 => {
            let fixed_header = check(stream.iter(), max_size)?;

            let packet = stream.split_to(fixed_header.frame_length());
            let packet_type = fixed_header.packet_type()?;

            return match packet_type{
                PacketType::PingReq => Ok(Packet::PingReq),
                PacketType::PingResp => Ok(Packet::PingResp),
                PacketType::Disconnect => Ok(Packet::Disconnect),
                _ =>  {
                    let packet = packet.freeze();
                    let packet = match packet_type {
                        PacketType::Connect => Packet::Connect(Connect::read(fixed_header, packet)?),
                        PacketType::ConnAck => Packet::ConnAck(ConnAck::read(fixed_header, packet)?),
                        PacketType::Publish => Packet::Publish(Publish::read(fixed_header, packet)?),
                        PacketType::PubAck => Packet::PubAck(PubAck::read(fixed_header, packet)?),
                        PacketType::PubRec => Packet::PubRec(PubRec::read(fixed_header, packet)?),
                        PacketType::PubRel => Packet::PubRel(PubRel::read(fixed_header, packet)?),
                        PacketType::PubComp => Packet::PubComp(PubComp::read(fixed_header, packet)?),
                        PacketType::Subscribe => Packet::Subscribe(Subscribe::read(fixed_header, packet)?),
                        PacketType::SubAck => Packet::SubAck(SubAck::read(fixed_header, packet)?),
                        PacketType::Unsubscribe => Packet::Unsubscribe(Unsubscribe::read(fixed_header, packet)?),
                        PacketType::UnsubAck => Packet::UnsubAck(UnsubAck::read(fixed_header, packet)?),
                        PacketType::PingReq => Packet::PingReq,
                        PacketType::PingResp => Packet::PingResp,
                        PacketType::Disconnect => Packet::Disconnect,
                    };

                    Ok(packet)
                }
            }       
        }
        Version::V5 => { todo!("V3.1.1 not supported yet"); }
    }
}

#[derive(Debug)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: String,
    pub qos: QoS
}

#[derive(Debug, Copy, Clone)]
pub enum Version {
    V31,
    V311,
    V5
}

impl FromStr for Version {
    type Err = String;

    fn from_str(version: &str) -> Result<Self, Self::Err> {
        match version {
            "3.1" => Ok(Version::V31),
            "3.1.1" => Ok(Version::V311),
            "5" => Ok(Version::V5),
            _ => Err("Invalid protocol version".to_string())
        }
    }
}
