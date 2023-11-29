use crate::encoding::{self, BitParsable, Parsable, Serializable};

use cookie_factory as cf;
use custom_debug_derive::Debug;
use nom::{
    bits, bits::complete::take as take_bits, combinator::map, multi::count,
    number::complete::be_u16, number::complete::be_u8, sequence::tuple,
};
use ux::{u1, u2};

use super::{ProtocolDataRate, RoutingScheme, RSSI};

#[derive(Debug, Clone, PartialEq)]
pub struct Repeater {
    /// Node ID of this repeater
    pub node_id: u8,
    // RSSI value of the acknowledgement frame measured by this repeater
    pub ack_rssi: Option<RSSI>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteFailLocation {
    pub last_functional_node_id: u8,
    pub first_non_functional_node_id: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Beam {
    #[debug(format = "250 ms")]
    Beam250ms,
    #[debug(format = "1000 ms")]
    Beam1000ms,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TransmitReport {
    /// Transmission time in ticks (multiples of 10ms)
    pub tx_ticks: u16,
    /// Transmit power used for the transmission in dBm
    pub tx_power: Option<i8>,
    /// Channel number used to transmit the data
    pub tx_channel_no: u8,
    /// The repeaters used in the route to the destination
    pub repeaters: Vec<Repeater>,
    /// State of the route resolution for the transmission attempt. Encoding is manufacturer specific. Z-Wave JS uses the Silicon Labs interpretation.
    pub routing_scheme: RoutingScheme,
    /// Transmission speed used in the last attempt
    pub route_speed: ProtocolDataRate,
    /// Whether the destination requires beaming to be reached, and if yes, which beam duration
    pub beam: Option<Beam>,
    /// How many routing attempts have been made to transmit the payload
    pub routing_attempts: u8,
    /// When a route failed, this indicated where the failure occurred along the route
    pub route_fail_location: Option<RouteFailLocation>,
    /// Measured noise floor during the outgoing transmission
    pub measured_noise_floor: Option<RSSI>,

    /// RSSI value of the acknowledgement frame measured by the controller
    pub ack_rssi: Option<RSSI>,
    /// Channel number the acknowledgement frame is received on
    pub ack_channel_no: Option<u8>,
    /// TX power in dBm used by the destination to transmit the ACK
    pub destination_ack_tx_power: Option<i8>,
    /// Measured RSSI of the acknowledgement frame received from the destination
    pub destination_ack_measured_rssi: Option<RSSI>,
    /// Noise floor measured by the destination during the ACK transmission
    pub destination_ack_measured_noise_floor: Option<RSSI>,
}

impl TransmitReport {
    // How to parse this depends on the Transmit status. ACK related fields are not parsed if the node did not ACK the frame.
    pub fn parse(i: encoding::Input, with_ack: bool) -> encoding::ParseResult<Self> {
        let (i, tx_ticks) = be_u16(i)?;
        let (i, num_repeaters) = be_u8(i)?;
        let (i, ack_rssi) = RSSI::parse(i)?;
        let (i, repeater_rssi) = count(RSSI::parse, 4usize)(i)?;
        let (i, ack_channel_no) = be_u8(i)?;
        let (i, tx_channel_no) = be_u8(i)?;
        let (i, routing_scheme) = RoutingScheme::parse(i)?;
        let (i, repeater_node_ids) = count(be_u8, 4usize)(i)?;
        let (i, (_reserved7, beam, _reserved43, route_speed)) = bits(tuple((
            u1::parse,
            map(take_bits(2usize), |x: u8| match x {
                1 => Some(Beam::Beam250ms),
                2 => Some(Beam::Beam1000ms),
                _ => None,
            }),
            u2::parse,
            <ProtocolDataRate as BitParsable>::parse,
        )))(i)?;
        let (i, routing_attempts) = be_u8(i)?;

        // TODO: Optionally parse those. They may not always be present
        let route_fail_location = None;
        let tx_power = None;
        let measured_noise_floor = None;
        let destination_ack_tx_power = None;
        let destination_ack_measured_rssi = None;
        let destination_ack_measured_noise_floor = None;

        let repeaters = repeater_node_ids
            .iter()
            .zip(repeater_rssi.iter())
            .map(|(node_id, rssi)| Repeater {
                node_id: *node_id,
                ack_rssi: if with_ack { Some(*rssi) } else { None },
            })
            .take(num_repeaters as usize)
            .collect();

        Ok((
            i,
            Self {
                tx_ticks,
                tx_power,
                tx_channel_no,
                repeaters,
                routing_scheme,
                route_speed,
                beam,
                routing_attempts,
                route_fail_location,
                measured_noise_floor,
                ack_rssi: if with_ack { Some(ack_rssi) } else { None },
                ack_channel_no: if with_ack { Some(ack_channel_no) } else { None },
                destination_ack_tx_power,
                destination_ack_measured_rssi,
                destination_ack_measured_noise_floor,
            },
        ))
    }
}

impl Serializable for TransmitReport {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cf::SerializeFn<W> + 'a {
        // use cf::{bytes::be_u8, sequence::tuple};
        move |_out| todo!("ERROR: TransmitReport::serialize() not implemented")
    }
}
