use crate::serialize::{self, Serializable};
use crate::parse::{
    self,
    bytes::be_u8,
    combinators::{context, map_res},
};
use crate::prelude::*;
use bytes::Bytes;
use proc_macros::TryFromRepr;

/// Complete list of function IDs for data messages.
/// IDs starting with FUNC_ID are straight from OZW and not implemented here yet.
/// IDs starting with UNKNOWN_FUNC are taken from openhab-zwave and not implemented here yet.
/// IDs starting with UNKNOWN_FUNC are also taken from https://github.com/yepher/RaZBerry/blob/master/README.md and not implemented yet
/// IDs ending with UNKNOWN_<hex-code> are reported by the stick but we don't know what they mean.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromRepr)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum FunctionType {
    GetSerialApiInitData = 0x02,
    SetApplicationNodeInformation = 0x03, // Set up the controller NIF prior to starting or joining a Z-Wave network

    ApplicationCommand = 0x04, // A message from another node

    GetControllerCapabilities = 0x05,
    SetSerialApiTimeouts = 0x06,
    GetSerialApiCapabilities = 0x07,

    SoftReset = 0x08,

    GetProtocolVersion = 0x09, // Used to request the Z-Wave Protocol version data (700 series)
    SerialApiStarted = 0x0a, // Sent by the controller after the serial API has been started (again)

    SerialApiSetup = 0x0b, // Configure the Serial API

    SetRFReceiveMode = 0x10, // Power the RF section of the stick down/up
    UNKNOWN_FUNC_SET_SLEEP_MODE = 0x11, // Set the CPU into sleep mode

    FUNC_ID_ZW_SEND_NODE_INFORMATION = 0x12, // Send Node Information Frame of the stick

    SendData = 0x13,          // Send data
    SendDataMulticast = 0x14, // Send data using multicast

    GetControllerVersion = 0x15,

    SendDataAbort = 0x16, // Abort sending data

    FUNC_ID_ZW_R_F_POWER_LEVEL_SET = 0x17, // Set RF Power level
    UNKNOWN_FUNC_SEND_DATA_META = 0x18,    // ??
    FUNC_ID_ZW_GET_RANDOM = 0x1c,          // Returns random data of variable length

    GetControllerId = 0x20, // Get Home ID and Controller Node ID

    UNKNOWN_FUNC_MEMORY_GET_BYTE = 0x21, // get a byte of memory
    UNKNOWN_FUNC_MEMORY_PUT_BYTE = 0x22, // write a byte of memory
    UNKNOWN_FUNC_MEMORY_GET_BUFFER = 0x23,
    UNKNOWN_FUNC_MEMORY_PUT_BUFFER = 0x24,

    EnterBootloader = 0x27, // Leave Serial API and enter bootloader (700+ series only). Enter Auto-Programming mode (500 series only).
    UNKNOWN_FUNC_UNKNOWN_0x28 = 0x28, // ??

    GetNVMId = 0x29,              // Returns information about the external NVM
    ExtNVMReadLongBuffer = 0x2a,  // Reads a buffer from the external NVM
    ExtNVMWriteLongBuffer = 0x2b, // Writes a buffer to the external NVM
    ExtNVMReadLongByte = 0x2c,    // Reads a byte from the external NVM
    ExtExtWriteLongByte = 0x2d,   // Writes a byte to the external NVM

    NVMOperations = 0x2e, // 700-series command to read and write from/to the external NVM

    UNKNOWN_FUNC_CLOCK_SET = 0x30,        // ??
    UNKNOWN_FUNC_CLOCK_GET = 0x31,        // ??
    UNKNOWN_FUNC_CLOCK_COMPARE = 0x32,    // ??
    UNKNOWN_FUNC_RTC_TIMER_CREATE = 0x33, // ??
    UNKNOWN_FUNC_RTC_TIMER_READ = 0x34,   // ??
    UNKNOWN_FUNC_RTC_TIMER_DELETE = 0x35, // ??
    UNKNOWN_FUNC_RTC_TIMER_CALL = 0x36,   // ??

    UNKNOWN_FUNC_ClearNetworkStats = 0x39,
    UNKNOWN_FUNC_GetNetworkStats = 0x3a,
    GetBackgroundRSSI = 0x3b, // request the most recent background RSSI levels detected
    UNKNOWN_FUNC_RemoveNodeIdFromNetwork = 0x3f,

    FUNC_ID_ZW_SET_LEARN_NODE_STATE = 0x40, // Not implemented

    GetNodeProtocolInfo = 0x41, // Get protocol info (baud rate, listening, etc.) for a given node
    HardReset = 0x42,           // Reset controller and node info to default (original) values

    FUNC_ID_ZW_NEW_CONTROLLER = 0x43, // Not implemented
    FUNC_ID_ZW_REPLICATION_COMMAND_COMPLETE = 0x44, // Replication send data complete
    FUNC_ID_ZW_REPLICATION_SEND_DATA = 0x45, // Replication send data
    AssignReturnRoute = 0x46, // Assign a return route from the source node to the destination node
    DeleteReturnRoute = 0x47, // Delete all return routes from the specified node
    RequestNodeNeighborUpdate = 0x48, // Ask the specified node to update its neighbors (then read them from the controller)
    ApplicationUpdateRequest = 0x49,  // Get a list of supported (and controller) command classes

    AddNodeToNetwork = 0x4a, // Control the addnode (or addcontroller) process...start, stop, etc.
    RemoveNodeFromNetwork = 0x4b, // Control the removenode (or removecontroller) process...start, stop, etc.

    FUNC_ID_ZW_CREATE_NEW_PRIMARY = 0x4c, // Control the createnewprimary process...start, stop, etc.
    FUNC_ID_ZW_CONTROLLER_CHANGE = 0x4d,  // Control the transferprimary process...start, stop, etc.

    AssignPriorityReturnRoute = 0x4f, // Assign a priority route between two nodes

    FUNC_ID_ZW_SET_LEARN_MODE = 0x50, // Put a controller into learn mode for replication/ receipt of configuration info
    AssignSUCReturnRoute = 0x51,      // Assign a return route to the SUC
    FUNC_ID_ZW_ENABLE_SUC = 0x52,     // Make a controller a Static Update Controller
    FUNC_ID_ZW_REQUEST_NETWORK_UPDATE = 0x53, // Network update for a SUC(?)
    SetSUCNodeId = 0x54, // Configure a static/bridge controller to be a SUC/SIS node (or not)
    DeleteSUCReturnRoute = 0x55, // Remove return routes to the SUC
    GetSUCNodeId = 0x56, // Try to retrieve a Static Update Controller node id (zero if no SUC present)

    UNKNOWN_FUNC_SEND_SUC_ID = 0x57,
    AssignPrioritySUCReturnRoute = 0x58, // Assign a priority route from a node to the SUC
    UNKNOWN_FUNC_REDISCOVERY_NEEDED = 0x59,

    FUNC_ID_ZW_REQUEST_NODE_NEIGHBOR_UPDATE_OPTIONS = 0x5a, // Allow options for request node neighbor update
    FUNC_ID_ZW_EXPLORE_REQUEST_INCLUSION = 0x5e,            // supports NWI

    RequestNodeInfo = 0x60, // Get info (supported command classes) for the specified node

    RemoveFailedNode = 0x61,  // Mark a specified node id as failed
    IsFailedNode = 0x62,      // Check to see if a specified node has failed
    ReplaceFailedNode = 0x63, // Replace a failed node with a new one that takes the same node ID

    UNKNOWN_FUNC_UNKNOWN_0x66 = 0x66, // ??
    UNKNOWN_FUNC_UNKNOWN_0x67 = 0x67, // ??

    UNKNOWN_FUNC_TIMER_START = 0x70,   // ??
    UNKNOWN_FUNC_TIMER_RESTART = 0x71, // ??
    UNKNOWN_FUNC_TIMER_CANCEL = 0x72,  // ??
    UNKNOWN_FUNC_TIMER_CALL = 0x73,    // ??

    FirmwareUpdateNVM = 0x78, // Access the NVM section for 500 series OTW firmware updates

    GetRoutingInfo = 0x80, // Get a specified node's neighbor information from the controller

    UNKNOWN_FUNC_GetTXCounter = 0x81,   // ??
    UNKNOWN_FUNC_ResetTXCounter = 0x82, // ??
    UNKNOWN_FUNC_StoreNodeInfo = 0x83,  // ??
    UNKNOWN_FUNC_StoreHomeId = 0x84,    // ??

    UNKNOWN_FUNC_LOCK_ROUTE_RESPONSE = 0x90,  // ??
    UNKNOWN_FUNC_SEND_DATA_ROUTE_DEMO = 0x91, // ??
    GetPriorityRoute = 0x92, // Get the route that is used as the first routing attempty when transmitting to a node
    SetPriorityRoute = 0x93, // Set the route that shall be used as the first routing attempty when transmitting to a node
    UNKNOWN_FUNC_SERIAL_API_TEST = 0x95, // ??
    UNKNOWN_FUNC_UNKNOWN_0x98 = 0x98, // ??

    FUNC_ID_SERIAL_API_SLAVE_NODE_INFO = 0xa0, // Set application virtual slave node information
    FUNC_ID_APPLICATION_SLAVE_COMMAND_HANDLER = 0xa1, // Slave command handler
    FUNC_ID_ZW_SEND_SLAVE_NODE_INFO = 0xa2,    // Send a slave node information message
    FUNC_ID_ZW_SEND_SLAVE_DATA = 0xa3,         // Send data from slave
    FUNC_ID_ZW_SET_SLAVE_LEARN_MODE = 0xa4,    // Enter slave learn mode
    FUNC_ID_ZW_GET_VIRTUAL_NODES = 0xa5,       // Return all virtual nodes
    FUNC_ID_ZW_IS_VIRTUAL_NODE = 0xa6,         // Virtual node test

    BridgeApplicationCommand = 0xa8, // A message from another node using the Bridge API
    SendDataBridge = 0xa9,           // Send data (Bridge API)
    SendDataMulticastBridge = 0xab,  // Send data using multicast (Bridge API)

    UNKNOWN_FUNC_UNKNOWN_0xB4 = 0xb4, // ??

    UNKNOWN_FUNC_WATCH_DOG_ENABLE = 0xb6,
    UNKNOWN_FUNC_WATCH_DOG_DISABLE = 0xb7,
    UNKNOWN_FUNC_WATCH_DOG_KICK = 0xb8,
    UNKNOWN_FUNC_UNKNOWN_0xB9 = 0xb9,      // ??
    UNKNOWN_FUNC_RF_POWERLEVEL_GET = 0xba, // Get RF Power level

    UNKNOWN_FUNC_GET_LIBRARY_TYPE = 0xbd,
    UNKNOWN_FUNC_SEND_TEST_FRAME = 0xbe,
    UNKNOWN_FUNC_GET_PROTOCOL_STATUS = 0xbf,

    FUNC_ID_ZW_SET_PROMISCUOUS_MODE = 0xd0, // Set controller into promiscuous mode to listen to all messages
    FUNC_ID_PROMISCUOUS_APPLICATION_COMMAND_HANDLER = 0xd1,

    UNKNOWN_FUNC_UNKNOWN_0xD2 = 0xd2, // ??
    UNKNOWN_FUNC_UNKNOWN_0xD3 = 0xd3, // ??
    UNKNOWN_FUNC_UNKNOWN_0xD4 = 0xd4, // ??

    Shutdown = 0xd9, // Instruct the Z-Wave API to shut down in order to safely remove the power

    UNKNOWN_FUNC_UNKNOWN_0xEF = 0xef, // ??

    // Special commands for Z-Wave.me sticks
    UNKNOWN_FUNC_ZMEFreqChange = 0xf2,
    UNKNOWN_FUNC_ZMERestore = 0xf3,
    UNKNOWN_FUNC_ZMEBootloaderFlash = 0xf4,
    UNKNOWN_FUNC_ZMECapabilities = 0xf5,
    UNKNOWN_FUNC_ZMESerialApiOptions = 0xf8,
}

impl Parsable for FunctionType {
    fn parse(i: &mut Bytes) -> parse::ParseResult<Self> {
        context("FunctionType", map_res(be_u8, FunctionType::try_from)).parse(i)
    }
}

impl Serializable for FunctionType {
    fn serialize(&self, output: &mut bytes::BytesMut) {
        use serialize::bytes::be_u8;
        be_u8(*self as u8).serialize(output);
    }
}
