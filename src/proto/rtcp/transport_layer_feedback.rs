use std::io::{Read, Write};

use crate::proto::common::sync_io::{ReadExt, WriteExt};
use crate::proto::error::ErrorKind;
use crate::proto::error::Error;
use crate::proto::rtcp::payload_specific_feedback::GenericNack;
use crate::proto::rtp::traits::RtcpPacketTrait;
use crate::proto::traits::{PacketTrait, ReadFrom, ReadPacket, Result, WritePacket, WriteTo};

use super::constants::*;
use super::feedback::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportLayerFeedbackPacket {
    Nack(GenericNack),
}
impl PacketTrait for TransportLayerFeedbackPacket {}
impl RtcpPacketTrait for TransportLayerFeedbackPacket {}
impl ReadFrom for TransportLayerFeedbackPacket {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let (fb_message_type, rest) = track_try_unwrap!(read_common(reader, RTCP_PACKET_TYPE_RTPFB).map_err(Error::from));
        match fb_message_type {
            RTPFB_MESSAGE_TYPE_NACK => {
                track_err!(GenericNack::read_from(&mut &rest[..])).map(From::from)
            }
            _ => track_panic!(
                ErrorKind::Unsupported,
                "Unknown feedback type: {}",
                fb_message_type
            ),
        }
    }
}
impl WriteTo for TransportLayerFeedbackPacket {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        match *self {
            TransportLayerFeedbackPacket::Nack(ref f) => {
                let payload = track_try_unwrap!(f.to_bytes().map_err(Error::from));
                track_err!(write_common(
                    writer,
                    RTCP_PACKET_TYPE_RTPFB,
                    RTPFB_MESSAGE_TYPE_NACK,
                    &payload
                ))
            }
        }
    }
}
impl From<GenericNack> for TransportLayerFeedbackPacket {
    fn from(f: GenericNack) -> Self {
        TransportLayerFeedbackPacket::Nack(f)
    }
}