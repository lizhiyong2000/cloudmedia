use std::io::{Read, Write};

use handy_async::sync_io::{ReadExt, WriteExt};

use crate::protocol::error::ErrorKind;
use crate::protocol::rtp::rtcp::payload_specific_feedback::GenericNack;
use crate::protocol::rtp::traits::RtcpPacketTrait;
use crate::protocol::traits::{PacketTrait, ReadFrom, ReadPacket, Result, WritePacket, WriteTo};

use super::constants::*;
use super::feedback::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RtcpTransportLayerFeedback {
    Nack(GenericNack),
}
impl PacketTrait for RtcpTransportLayerFeedback {}
impl RtcpPacketTrait for RtcpTransportLayerFeedback {}
impl ReadFrom for RtcpTransportLayerFeedback {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let (fb_message_type, rest) = track_try!(read_common(reader, RTCP_PACKET_TYPE_RTPFB));
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
impl WriteTo for RtcpTransportLayerFeedback {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        match *self {
            RtcpTransportLayerFeedback::Nack(ref f) => {
                let payload = track_try!(f.to_bytes());
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
impl From<GenericNack> for RtcpTransportLayerFeedback {
    fn from(f: GenericNack) -> Self {
        RtcpTransportLayerFeedback::Nack(f)
    }
}