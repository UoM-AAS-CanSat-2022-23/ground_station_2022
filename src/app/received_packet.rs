use crate::telemetry::Telemetry;
use crate::xbee::{RxPacket, TxStatus, XbeePacket};
use std::fmt;

#[derive(Debug, Clone)]
pub enum ReceivedPacket {
    // an incoming telemetry packet
    Telemetry {
        // the packet containing the telemetry
        packet: XbeePacket,
        // the parsed RxPacket
        frame: RxPacket,
        // the parsed telemetry
        telem: Telemetry,
    },

    // an incoming packet that parsed correctly but couldn't be parsed as telemetry
    Received {
        packet: XbeePacket,
        frame: RxPacket,
    },

    // status information for the packet with the given frame ID
    Status {
        // the packet containing the TxStatus
        packet: XbeePacket,
        // the parsed TxStatus
        tx_status: TxStatus,
    },

    // an incoming packet which had a good frame ID but parsing the inner frame failed
    InvalidFrame(XbeePacket),

    // an incoming packet which had an unrecognised frame type
    Unrecognised(XbeePacket),

    // an incoming packet that was unparseable
    Invalid(Vec<u8>),
}

impl From<&[u8]> for ReceivedPacket {
    fn from(raw_packet: &[u8]) -> Self {
        // first try and parse it as an XbeePacket
        let xbp = match XbeePacket::decode(raw_packet) {
            Ok(xbp) => xbp,
            Err(e) => {
                tracing::warn!("Failed to parse radio data - {e:?}");
                return Self::Invalid(raw_packet.to_vec());
            }
        };

        // then match on the frame type
        let received_data = match xbp.frame_type {
            // RxPacket frame type
            0x81 => match RxPacket::try_from(xbp.clone()) {
                Ok(rxp) => rxp,
                Err(e) => {
                    tracing::warn!("Failed to parse incoming RxPacket - {e:?}");
                    return Self::InvalidFrame(xbp);
                }
            },
            // TxStatus frame type
            0x89 => {
                match TxStatus::try_from(xbp.clone()) {
                    // if the packet parsed well, return the status and the frame ID
                    Ok(status) => {
                        return Self::Status {
                            packet: xbp,
                            tx_status: status,
                        }
                    }
                    // otherwise log an
                    Err(e) => {
                        tracing::warn!("Failed to parse incoming TxStatus - {e:?}");
                        return Self::InvalidFrame(xbp);
                    }
                }
            }
            _ => {
                return Self::Unrecognised(xbp);
            }
        };

        // get a UTF8 string from the sent data
        let string_data = match String::from_utf8(received_data.data.clone()) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Incoming RxPacket contained invalid UTF8 data - {e:?}");
                return Self::Received {
                    packet: xbp,
                    frame: received_data,
                };
            }
        };

        // parse the string as telemetry
        tracing::debug!("string_data={string_data:?}");
        match string_data.parse() {
            Ok(telem) => Self::Telemetry {
                packet: xbp,
                frame: received_data,
                telem,
            },
            Err(e) => {
                tracing::warn!("Failed to parse RxPacket data as telemetry - {e:?}");
                Self::Received {
                    packet: xbp,
                    frame: received_data,
                }
            }
        }
    }
}

impl fmt::Display for ReceivedPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReceivedPacket::Telemetry { telem, .. } => {
                write!(f, "Telemetry - {telem}")
            }
            ReceivedPacket::Received { frame, .. } => {
                write!(f, "{frame}")
            }
            ReceivedPacket::Status { tx_status, .. } => {
                write!(f, "{tx_status}")
            }
            ReceivedPacket::InvalidFrame(xbp) => {
                write!(f, "Invalid RxFrame - {xbp}")
            }
            ReceivedPacket::Unrecognised(xbp) => {
                write!(f, "Unrecognised frame type - {xbp}")
            }
            ReceivedPacket::Invalid(data) => {
                write!(
                    f,
                    "Invalid Data - {data:02X?} - {:?}",
                    String::from_utf8_lossy(data)
                )
            }
        }
    }
}
