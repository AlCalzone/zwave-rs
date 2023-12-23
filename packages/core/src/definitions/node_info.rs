use nom::{
    bits,
    bits::complete::bool,
    combinator::{cond, opt, map_res},
    number::complete::be_u8,
    sequence::tuple,
};
use ux::{u1, u2, u5};

use crate::{
    definitions::node_type,
    encoding::{self, BitParsable},
    prelude::Parsable,
};

use super::{protocol_version, BasicDeviceType, Beam, DataRate, NodeType, ProtocolVersion};

#[derive(Debug, Clone, PartialEq)]
pub struct NodeInformationProtocolData {
    /// Whether this node is always listening
    listening: bool,
    /// Whether this node is frequently listening, and if yes, with which interval
    frequent_listening: Option<Beam>,
    /// Whether the node supports routing/forwarding messages
    routing: bool,
    /// Which data rates the node supports
    supported_data_rates: Vec<DataRate>,
    /// The protocol version this node implements
    protocol_version: ProtocolVersion,
    /// Whether this node supports additional CCs besides the mandatory minimum
    optional_functionality: bool,
    /// Whether this node is a controller (can calculate routes) or an end node (relies on route info)
    node_type: NodeType,
    /// Whether this node supports secure communication (official Host API specs) / legacy network security (legacy Host API specs).
    supports_security: bool,
    // Whether the node can wake up FLiRS nodes
    beaming: bool,
    /// The basic device type of this node. Only present if the node is a controller
    basic_device_type: Option<BasicDeviceType>,
    /// Which generic device class is implemented by this node
    generic_device_class: u8,
    /// Which specific device class is implemented by this node
    specific_device_class: Option<u8>,
}

impl Parsable for NodeInformationProtocolData {
    fn parse(i: &[u8]) -> encoding::ParseResult<Self> {
        let (i, (listening, routing, _reserved5, speed_40k, speed_9k6, protocol_version)) =
            bits(tuple((
                bool,
                bool,
                u1::parse,
                bool,
                bool,
                <ProtocolVersion as BitParsable>::parse,
            )))(i)?;

        let (
            i,
            (
                optional_functionality,
                frequent_listening,
                beaming,
                end_node,
                has_specific_device_class,
                controller,
                supports_security,
            ),
        ) = bits(tuple((
            bool,
            Beam::parse_opt,
            bool,
            bool,
            bool,
            bool,
            bool,
        )))(i)?;

        let (i, (_reserved73, _reserved21, speed_100k)) =
            bits(tuple((u5::parse, u2::parse, bool)))(i)?;

        let (i, basic_device_type) = cond(controller, BasicDeviceType::parse)(i)?;
        let (i, generic_device_class) = be_u8(i)?;
        let (i, specific_device_class) = cond(has_specific_device_class, be_u8)(i)?;

        let mut supported_data_rates = Vec::new();
        if speed_100k {
            supported_data_rates.push(DataRate::DataRate_100k);
        }
        if speed_40k {
            supported_data_rates.push(DataRate::DataRate_40k);
        }
        if speed_9k6 {
            supported_data_rates.push(DataRate::DataRate_9k6);
        }

        Ok((i, Self {
            listening,
            frequent_listening,
            routing,
            supported_data_rates,
            protocol_version,
            optional_functionality,
            node_type: if end_node {
                NodeType::EndNode
            } else {
                NodeType::Controller
            },
            supports_security,
            beaming,
            basic_device_type,
            generic_device_class,
            specific_device_class,
        }))
    }
}
