
use nom::{bits, bits::complete::bool, combinator::cond, number::complete::be_u8, sequence::tuple};
use ux::{u1, u2, u5};

use crate::{
    encoding::{self, parsers, BitParsable},
    prelude::Parsable,
};

use super::{BasicDeviceType, Beam, CommandClasses, DataRate, NodeType, ProtocolVersion};

#[derive(Debug, Clone, PartialEq)]
pub struct NodeInformationProtocolData {
    /// Whether this node is always listening
    pub listening: bool,
    /// Whether this node is frequently listening, and if yes, with which interval
    pub frequent_listening: Option<Beam>,
    /// Whether the node supports routing/forwarding messages
    pub routing: bool,
    /// Which data rates the node supports
    pub supported_data_rates: Vec<DataRate>,
    /// The protocol version this node implements
    pub protocol_version: ProtocolVersion,
    /// Whether this node supports additional CCs besides the mandatory minimum
    pub optional_functionality: bool,
    /// Whether this node is a controller (can calculate routes) or an end node (relies on route info)
    pub node_type: NodeType,
    /// Whether this node supports secure communication (official Host API specs) / legacy network security (legacy Host API specs).
    pub supports_security: bool,
    /// Whether the node can wake up FLiRS nodes
    pub beaming: bool,
    /// The basic device type of this node. Only present if the node is a controller
    pub basic_device_type: Option<BasicDeviceType>,
    /// Which generic device class is implemented by this node
    pub generic_device_class: u8,
    /// Which specific device class is implemented by this node
    pub specific_device_class: Option<u8>,
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
        ) = bits(tuple((bool, Beam::parse_opt, bool, bool, bool, bool, bool)))(i)?;

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

        Ok((
            i,
            Self {
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
            },
        ))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeInformationApplicationData {
    /// The basic device type of this node
    pub basic_device_type: BasicDeviceType,
    /// Which generic device class is implemented by this node
    pub generic_device_class: u8,
    /// Which specific device class is implemented by this node
    pub specific_device_class: u8,
    /// Which command classes are supported by this node
    pub supported_command_classes: Vec<CommandClasses>,
}

impl Parsable for NodeInformationApplicationData {
    fn parse(i: encoding::Input) -> encoding::ParseResult<Self> {
        // The specs call this CC list length, but this includes the device class bytes
        let (i, remaining_len) = be_u8(i)?;
        let (i, basic_device_type) = BasicDeviceType::parse(i)?;
        let (i, generic_device_class) = be_u8(i)?;
        let (i, specific_device_class) = be_u8(i)?;
        let (i, supported_command_classes) =
            parsers::fixed_length_cc_list_only_supported(i, (remaining_len - 3) as usize)?;

        Ok((
            i,
            Self {
                basic_device_type,
                generic_device_class,
                specific_device_class,
                supported_command_classes,
            },
        ))
    }
}
