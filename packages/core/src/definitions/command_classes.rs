use crate::prelude::{Parsable, Serializable};
use derive_try_from_primitive::TryFromPrimitive;
use enum_iterator::Sequence;
use nom::{
    combinator::{map, peek},
    number::complete::{be_u16, be_u8},
};
use std::fmt::Display;

pub const COMMAND_CLASS_SUPPORT_CONTROL_MARK: u8 = 0xef;

#[derive(Debug, Clone, Copy, PartialEq, Sequence, TryFromPrimitive)]
#[repr(u16)]
pub enum CommandClasses {
    AlarmSensor = 0x9c,
    AlarmSilence = 0x9d,
    AllSwitch = 0x27,
    AntiTheft = 0x5d,
    AntiTheftUnlock = 0x7e,
    ApplicationCapability = 0x57,
    ApplicationStatus = 0x22,
    Association = 0x85,
    AssociationCommandConfiguration = 0x9b,
    AssociationGroupInformation = 0x59,
    Authentication = 0xa1,
    AuthenticationMediaWrite = 0xa2,
    BarrierOperator = 0x66,
    Basic = 0x20,
    BasicTariffInformation = 0x36,
    BasicWindowCovering = 0x50,
    Battery = 0x80,
    BinarySensor = 0x30,
    BinarySwitch = 0x25,
    BinaryToggleSwitch = 0x28,
    ClimateControlSchedule = 0x46,
    CentralScene = 0x5b,
    Clock = 0x81,
    ColorSwitch = 0x33,
    Configuration = 0x70,
    ControllerReplication = 0x21,
    CRC16Encapsulation = 0x56,
    DemandControlPlanConfiguration = 0x3a,
    DemandControlPlanMonitor = 0x3b,
    DeviceResetLocally = 0x5a,
    DoorLock = 0x62,
    DoorLockLogging = 0x4c,
    EnergyProduction = 0x90,
    EntryControl = 0x6f,
    FirmwareUpdateMetaData = 0x7a,
    GenericSchedule = 0xa3,
    GeographicLocation = 0x8c,
    GroupingName = 0x7b,
    Hail = 0x82,
    HRVStatus = 0x37,
    HRVControl = 0x39,
    HumidityControlMode = 0x6d,
    HumidityControlOperatingState = 0x6e,
    HumidityControlSetpoint = 0x64,
    InclusionController = 0x74,
    Indicator = 0x87,
    IPAssociation = 0x5c,
    IPConfiguration = 0x9a,
    IRRepeater = 0xa0,
    Irrigation = 0x6b,
    Language = 0x89,
    Lock = 0x76,
    Mailbox = 0x69,
    ManufacturerProprietary = 0x91,
    ManufacturerSpecific = 0x72,
    Meter = 0x32,
    MeterTableConfiguration = 0x3c,
    MeterTableMonitor = 0x3d,
    MeterTablePushConfiguration = 0x3e,
    MoveToPositionWindowCovering = 0x51,
    MultiChannel = 0x60,
    MultiChannelAssociation = 0x8e,
    MultiCommand = 0x8f,
    MultilevelSensor = 0x31,
    MultilevelSwitch = 0x26,
    MultilevelToggleSwitch = 0x29,
    NetworkManagementBasicNode = 0x4d,
    NetworkManagementInclusion = 0x34,
    NetworkManagementInstallationAndMaintenance = 0x67,
    NetworkManagementPrimary = 0x54,
    NetworkManagementProxy = 0x52,
    NoOperation = 0x00,
    NodeNamingAndLocation = 0x77,
    NodeProvisioning = 0x78,
    Notification = 0x71,
    Powerlevel = 0x73,
    Prepayment = 0x3f,
    PrepaymentEncapsulation = 0x41,
    Proprietary = 0x88,
    Protection = 0x75,
    PulseMeter = 0x35,
    RateTableConfiguration = 0x48,
    RateTableMonitor = 0x49,
    RemoteAssociationActivation = 0x7c,
    RemoteAssociationConfiguration = 0x7d,
    SceneActivation = 0x2b,
    SceneActuatorConfiguration = 0x2c,
    SceneControllerConfiguration = 0x2d,
    Schedule = 0x53,
    ScheduleEntryLock = 0x4e,
    ScreenAttributes = 0x93,
    ScreenMetaData = 0x92,
    Security = 0x98,
    Security2 = 0x9f,
    SecurityMark = 0xf100,
    SensorConfiguration = 0x9e,
    SimpleAVControl = 0x94,
    SoundSwitch = 0x79,
    Supervision = 0x6c,
    TariffTableConfiguration = 0x4a,
    TariffTableMonitor = 0x4b,
    ThermostatFanMode = 0x44,
    ThermostatFanState = 0x45,
    ThermostatMode = 0x40,
    ThermostatOperatingState = 0x42,
    ThermostatSetback = 0x47,
    ThermostatSetpoint = 0x43,
    Time = 0x8a,
    TimeParameters = 0x8b,
    TransportService = 0x55,
    UserCode = 0x63,
    Version = 0x86,
    WakeUp = 0x84,
    WindowCovering = 0x6a,
    ZIP = 0x23,
    ZIP6LoWPAN = 0x4f,
    ZIPGateway = 0x5f,
    ZIPNamingAndLocation = 0x68,
    ZIPND = 0x58,
    ZIPPortal = 0x61,
    ZWavePlusInfo = 0x5e,
    // Internal CC which is not used directly by applications
    ZWaveProtocol = 0x01,
}

impl CommandClasses {
    pub fn is_extended_cc(&self) -> bool {
        *self as u16 > 0xf1u16
    }

    pub fn is_extended<T: Into<u16>>(val: T) -> bool {
        val.into() > 0xf1u16
    }

    /// Returns an iterator over all defined command classes
    pub fn all_ccs() -> impl Iterator<Item = Self> {
        enum_iterator::all::<Self>()
    }

    /// Defines which CCs are considered Application CCs
    pub fn application_ccs() -> impl Iterator<Item = Self> {
        [
            CommandClasses::AlarmSensor,
            CommandClasses::AlarmSilence,
            CommandClasses::AllSwitch,
            CommandClasses::AntiTheft,
            CommandClasses::BarrierOperator,
            CommandClasses::Basic,
            CommandClasses::BasicTariffInformation,
            CommandClasses::BasicWindowCovering,
            CommandClasses::BinarySensor,
            CommandClasses::BinarySwitch,
            CommandClasses::BinaryToggleSwitch,
            CommandClasses::ClimateControlSchedule,
            CommandClasses::CentralScene,
            CommandClasses::Clock,
            CommandClasses::ColorSwitch,
            CommandClasses::Configuration,
            CommandClasses::ControllerReplication,
            CommandClasses::DemandControlPlanConfiguration,
            CommandClasses::DemandControlPlanMonitor,
            CommandClasses::DoorLock,
            CommandClasses::DoorLockLogging,
            CommandClasses::EnergyProduction,
            CommandClasses::EntryControl,
            CommandClasses::GenericSchedule,
            CommandClasses::GeographicLocation,
            CommandClasses::HRVStatus,
            CommandClasses::HRVControl,
            CommandClasses::HumidityControlMode,
            CommandClasses::HumidityControlOperatingState,
            CommandClasses::HumidityControlSetpoint,
            CommandClasses::IRRepeater,
            CommandClasses::Irrigation,
            CommandClasses::Language,
            CommandClasses::Lock,
            CommandClasses::ManufacturerProprietary,
            CommandClasses::Meter,
            CommandClasses::MeterTableConfiguration,
            CommandClasses::MeterTableMonitor,
            CommandClasses::MeterTablePushConfiguration,
            CommandClasses::MoveToPositionWindowCovering,
            CommandClasses::MultilevelSensor,
            CommandClasses::MultilevelSwitch,
            CommandClasses::MultilevelToggleSwitch,
            CommandClasses::Notification,
            CommandClasses::Prepayment,
            CommandClasses::PrepaymentEncapsulation,
            CommandClasses::Proprietary,
            CommandClasses::Protection,
            CommandClasses::PulseMeter,
            CommandClasses::RateTableConfiguration,
            CommandClasses::RateTableMonitor,
            CommandClasses::SceneActivation,
            CommandClasses::SceneActuatorConfiguration,
            CommandClasses::SceneControllerConfiguration,
            CommandClasses::Schedule,
            CommandClasses::ScheduleEntryLock,
            CommandClasses::ScreenAttributes,
            CommandClasses::ScreenMetaData,
            CommandClasses::SensorConfiguration,
            CommandClasses::SimpleAVControl,
            CommandClasses::SoundSwitch,
            CommandClasses::TariffTableConfiguration,
            CommandClasses::TariffTableMonitor,
            CommandClasses::ThermostatFanMode,
            CommandClasses::ThermostatFanState,
            CommandClasses::ThermostatMode,
            CommandClasses::ThermostatOperatingState,
            CommandClasses::ThermostatSetback,
            CommandClasses::ThermostatSetpoint,
            CommandClasses::UserCode,
            CommandClasses::WindowCovering,
        ]
        .into_iter()
    }

    pub fn non_application_ccs() -> impl Iterator<Item = Self> {
        // 	allCCs.filter((cc) => !applicationCCs.includes(cc)),
        let application_ccs: Vec<_> = Self::application_ccs().collect();
        Self::all_ccs().filter(move |cc| !application_ccs.contains(cc))
    }

    /// Defines which CCs are considered Actuator CCs
    pub fn actuator_ccs() -> impl Iterator<Item = Self> {
        [
            CommandClasses::BarrierOperator,
            CommandClasses::BinarySwitch,
            CommandClasses::ColorSwitch,
            CommandClasses::DoorLock,
            CommandClasses::MultilevelSwitch,
            CommandClasses::SimpleAVControl,
            CommandClasses::SoundSwitch,
            CommandClasses::ThermostatSetpoint,
            CommandClasses::ThermostatMode,
            CommandClasses::WindowCovering,
        ]
        .into_iter()
    }

    /// Defines which CCs are considered Sensor CCs
    pub fn sensor_ccs() -> impl Iterator<Item = Self> {
        [
            CommandClasses::AlarmSensor,
            CommandClasses::Battery,
            CommandClasses::BinarySensor,
            CommandClasses::EnergyProduction,
            CommandClasses::Meter,
            CommandClasses::MultilevelSensor,
            CommandClasses::Notification, // For pull nodes
            CommandClasses::PulseMeter,
        ]
        .into_iter()
    }

    /// Defines which CCs are considered Encapsulation CCs
    pub fn encapsulation_ccs() -> impl Iterator<Item = Self> {
        [
            CommandClasses::CRC16Encapsulation,
            CommandClasses::MultiChannel,
            CommandClasses::MultiCommand,
            CommandClasses::Security,
            CommandClasses::Security2,
            CommandClasses::Supervision,
            CommandClasses::TransportService,
        ]
        .into_iter()
    }

    /// Defines which CCs are considered Management CCs
    pub fn management_ccs() -> impl Iterator<Item = Self> {
        [
            CommandClasses::ApplicationCapability,
            CommandClasses::ApplicationStatus,
            CommandClasses::Association,
            CommandClasses::AssociationCommandConfiguration,
            CommandClasses::AssociationGroupInformation,
            // Battery is in the Management CC specs, but we consider it a Sensor CC
            CommandClasses::DeviceResetLocally,
            CommandClasses::FirmwareUpdateMetaData,
            CommandClasses::GroupingName,
            CommandClasses::Hail,
            CommandClasses::Indicator,
            CommandClasses::IPAssociation,
            CommandClasses::ManufacturerSpecific,
            CommandClasses::MultiChannelAssociation,
            CommandClasses::NodeNamingAndLocation,
            CommandClasses::RemoteAssociationActivation,
            CommandClasses::RemoteAssociationConfiguration,
            CommandClasses::Time,
            CommandClasses::TimeParameters,
            CommandClasses::Version,
            CommandClasses::WakeUp,
            CommandClasses::ZIPNamingAndLocation,
            CommandClasses::ZWavePlusInfo,
        ]
        .into_iter()
    }
}

impl Display for CommandClasses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandClasses::AlarmSensor => write!(f, "Alarm Sensor"),
            CommandClasses::AlarmSilence => write!(f, "Alarm Silence"),
            CommandClasses::AllSwitch => write!(f, "All Switch"),
            CommandClasses::AntiTheft => write!(f, "Anti-Theft"),
            CommandClasses::AntiTheftUnlock => write!(f, "Anti-Theft Unlock"),
            CommandClasses::ApplicationCapability => write!(f, "Application Capability"),
            CommandClasses::ApplicationStatus => write!(f, "Application Status"),
            CommandClasses::Association => write!(f, "Association"),
            CommandClasses::AssociationCommandConfiguration => {
                write!(f, "Association Command Configuration")
            }
            CommandClasses::AssociationGroupInformation => {
                write!(f, "Association Group Information")
            }
            CommandClasses::Authentication => write!(f, "Authentication"),
            CommandClasses::AuthenticationMediaWrite => write!(f, "Authentication Media Write"),
            CommandClasses::BarrierOperator => write!(f, "Barrier Operator"),
            CommandClasses::Basic => write!(f, "Basic"),
            CommandClasses::BasicTariffInformation => write!(f, "Basic Tariff Information"),
            CommandClasses::BasicWindowCovering => write!(f, "Basic Window Covering"),
            CommandClasses::Battery => write!(f, "Battery"),
            CommandClasses::BinarySensor => write!(f, "Binary Sensor"),
            CommandClasses::BinarySwitch => write!(f, "Binary Switch"),
            CommandClasses::BinaryToggleSwitch => write!(f, "Binary Toggle Switch"),
            CommandClasses::ClimateControlSchedule => write!(f, "Climate Control Schedule"),
            CommandClasses::CentralScene => write!(f, "Central Scene"),
            CommandClasses::Clock => write!(f, "Clock"),
            CommandClasses::ColorSwitch => write!(f, "Color Switch"),
            CommandClasses::Configuration => write!(f, "Configuration"),
            CommandClasses::ControllerReplication => write!(f, "Controller Replication"),
            CommandClasses::CRC16Encapsulation => write!(f, "CRC-16 Encapsulation"),
            CommandClasses::DemandControlPlanConfiguration => {
                write!(f, "Demand Control Plan Configuration")
            }
            CommandClasses::DemandControlPlanMonitor => write!(f, "Demand Control Plan Monitor"),
            CommandClasses::DeviceResetLocally => write!(f, "Device Reset Locally"),
            CommandClasses::DoorLock => write!(f, "Door Lock"),
            CommandClasses::DoorLockLogging => write!(f, "Door Lock Logging"),
            CommandClasses::EnergyProduction => write!(f, "Energy Production"),
            CommandClasses::EntryControl => write!(f, "Entry Control"),
            CommandClasses::FirmwareUpdateMetaData => write!(f, "Firmware Update Meta Data"),
            CommandClasses::GenericSchedule => write!(f, "Generic Schedule"),
            CommandClasses::GeographicLocation => write!(f, "Geographic Location"),
            CommandClasses::GroupingName => write!(f, "Grouping Name"),
            CommandClasses::Hail => write!(f, "Hail"),
            CommandClasses::HRVStatus => write!(f, "HRV Status"),
            CommandClasses::HRVControl => write!(f, "HRV Control"),
            CommandClasses::HumidityControlMode => write!(f, "Humidity Control Mode"),
            CommandClasses::HumidityControlOperatingState => {
                write!(f, "Humidity Control Operating State")
            }
            CommandClasses::HumidityControlSetpoint => write!(f, "Humidity Control Setpoint"),
            CommandClasses::InclusionController => write!(f, "Inclusion Controller"),
            CommandClasses::Indicator => write!(f, "Indicator"),
            CommandClasses::IPAssociation => write!(f, "IP Association"),
            CommandClasses::IPConfiguration => write!(f, "IP Configuration"),
            CommandClasses::IRRepeater => write!(f, "IR Repeater"),
            CommandClasses::Irrigation => write!(f, "Irrigation"),
            CommandClasses::Language => write!(f, "Language"),
            CommandClasses::Lock => write!(f, "Lock"),
            CommandClasses::Mailbox => write!(f, "Mailbox"),
            CommandClasses::ManufacturerProprietary => write!(f, "Manufacturer Proprietary"),
            CommandClasses::ManufacturerSpecific => write!(f, "Manufacturer Specific"),
            CommandClasses::Meter => write!(f, "Meter"),
            CommandClasses::MeterTableConfiguration => write!(f, "Meter Table Configuration"),
            CommandClasses::MeterTableMonitor => write!(f, "Meter Table Monitor"),
            CommandClasses::MeterTablePushConfiguration => {
                write!(f, "Meter Table Push Configuration")
            }
            CommandClasses::MoveToPositionWindowCovering => {
                write!(f, "Move To Position Window Covering")
            }
            CommandClasses::MultiChannel => write!(f, "Multi Channel"),
            CommandClasses::MultiChannelAssociation => write!(f, "Multi Channel Association"),
            CommandClasses::MultiCommand => write!(f, "Multi Command"),
            CommandClasses::MultilevelSensor => write!(f, "Multilevel Sensor"),
            CommandClasses::MultilevelSwitch => write!(f, "Multilevel Switch"),
            CommandClasses::MultilevelToggleSwitch => write!(f, "Multilevel Toggle Switch"),
            CommandClasses::NetworkManagementBasicNode => {
                write!(f, "Network Management Basic Node")
            }
            CommandClasses::NetworkManagementInclusion => write!(f, "Network Management Inclusion"),
            CommandClasses::NetworkManagementInstallationAndMaintenance => {
                write!(f, "Network Management Installation and Maintenance")
            }
            CommandClasses::NetworkManagementPrimary => write!(f, "Network Management Primary"),
            CommandClasses::NetworkManagementProxy => write!(f, "Network Management Proxy"),
            CommandClasses::NoOperation => write!(f, "No Operation"),
            CommandClasses::NodeNamingAndLocation => write!(f, "Node Naming and Location"),
            CommandClasses::NodeProvisioning => write!(f, "Node Provisioning"),
            CommandClasses::Notification => write!(f, "Notification"),
            CommandClasses::Powerlevel => write!(f, "Powerlevel"),
            CommandClasses::Prepayment => write!(f, "Prepayment"),
            CommandClasses::PrepaymentEncapsulation => write!(f, "Prepayment Encapsulation"),
            CommandClasses::Proprietary => write!(f, "Proprietary"),
            CommandClasses::Protection => write!(f, "Protection"),
            CommandClasses::PulseMeter => write!(f, "Pulse Meter"),
            CommandClasses::RateTableConfiguration => write!(f, "Rate Table Configuration"),
            CommandClasses::RateTableMonitor => write!(f, "Rate Table Monitor"),
            CommandClasses::RemoteAssociationActivation => {
                write!(f, "Remote Association Activation")
            }
            CommandClasses::RemoteAssociationConfiguration => {
                write!(f, "Remote Association Configuration")
            }
            CommandClasses::SceneActivation => write!(f, "Scene Activation"),
            CommandClasses::SceneActuatorConfiguration => write!(f, "Scene Actuator Configuration"),
            CommandClasses::SceneControllerConfiguration => {
                write!(f, "Scene Controller Configuration")
            }
            CommandClasses::Schedule => write!(f, "Schedule"),
            CommandClasses::ScheduleEntryLock => write!(f, "Schedule Entry Lock"),
            CommandClasses::ScreenAttributes => write!(f, "Screen Attributes"),
            CommandClasses::ScreenMetaData => write!(f, "Screen Meta Data"),
            CommandClasses::Security => write!(f, "Security"),
            CommandClasses::Security2 => write!(f, "Security 2"),
            CommandClasses::SecurityMark => write!(f, "Security Mark"),
            CommandClasses::SensorConfiguration => write!(f, "Sensor Configuration"),
            CommandClasses::SimpleAVControl => write!(f, "Simple AV Control"),
            CommandClasses::SoundSwitch => write!(f, "Sound Switch"),
            CommandClasses::Supervision => write!(f, "Supervision"),
            CommandClasses::TariffTableConfiguration => write!(f, "Tariff Table Configuration"),
            CommandClasses::TariffTableMonitor => write!(f, "Tariff Table Monitor"),
            CommandClasses::ThermostatFanMode => write!(f, "Thermostat Fan Mode"),
            CommandClasses::ThermostatFanState => write!(f, "Thermostat Fan State"),
            CommandClasses::ThermostatMode => write!(f, "Thermostat Mode"),
            CommandClasses::ThermostatOperatingState => write!(f, "Thermostat Operating State"),
            CommandClasses::ThermostatSetback => write!(f, "Thermostat Setback"),
            CommandClasses::ThermostatSetpoint => write!(f, "Thermostat Setpoint"),
            CommandClasses::Time => write!(f, "Time"),
            CommandClasses::TimeParameters => write!(f, "Time Parameters"),
            CommandClasses::TransportService => write!(f, "Transport Service"),
            CommandClasses::UserCode => write!(f, "User Code"),
            CommandClasses::Version => write!(f, "Version"),
            CommandClasses::WakeUp => write!(f, "Wake Up"),
            CommandClasses::WindowCovering => write!(f, "Window Covering"),
            CommandClasses::ZIP => write!(f, "Z/IP"),
            CommandClasses::ZIP6LoWPAN => write!(f, "Z/IP 6LoWPAN"),
            CommandClasses::ZIPGateway => write!(f, "Z/IP Gateway"),
            CommandClasses::ZIPNamingAndLocation => write!(f, "Z/IP Naming and Location"),
            CommandClasses::ZIPND => write!(f, "Z/IP ND"),
            CommandClasses::ZIPPortal => write!(f, "Z/IP Portal"),
            CommandClasses::ZWavePlusInfo => write!(f, "Z-Wave Plus Info"),
            CommandClasses::ZWaveProtocol => write!(f, "Z-Wave Protocol"),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct CommandClassInfo {
    supported: bool,
    controlled: bool,
    secure: bool,
    version: u8,
}

impl CommandClassInfo {
    pub fn new(supported: bool, controlled: bool, secure: bool, version: u8) -> CommandClassInfo {
        CommandClassInfo {
            supported,
            controlled,
            secure,
            version,
        }
    }

    pub fn supported(&self) -> bool {
        self.supported
    }

    pub fn set_supported(&mut self, supported: bool) {
        self.supported = supported;
    }

    pub fn controlled(&self) -> bool {
        self.controlled
    }

    pub fn set_controlled(&mut self, controlled: bool) {
        self.controlled = controlled;
    }

    pub fn secure(&self) -> bool {
        self.secure
    }

    pub fn set_secure(&mut self, secure: bool) {
        self.secure = secure;
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn set_version(&mut self, version: u8) {
        self.version = version;
    }
}

#[test]
fn test_non_application_ccs() {
    let nac = CommandClasses::non_application_ccs().collect::<Vec<_>>();
    assert!(nac.contains(&CommandClasses::Association));
}

impl Parsable for CommandClasses {
    fn parse(i: crate::encoding::Input) -> crate::prelude::ParseResult<Self> {
        let (i, cc_id) = peek(be_u8)(i)?;
        // FIXME: Support unknown CCs
        let (i, cc) = if CommandClasses::is_extended(cc_id) {
            map(be_u16, |x| CommandClasses::try_from(x).unwrap())(i)?
        } else {
            map(be_u8, |x| CommandClasses::try_from(x as u16).unwrap())(i)?
        };
        Ok((i, cc))
    }
}

impl Serializable for CommandClasses {
    fn serialize<'a, W: std::io::Write + 'a>(&'a self) -> impl cookie_factory::SerializeFn<W> + 'a {
        use cookie_factory::bytes::{be_u16, be_u8};
        move |out| {
            if self.is_extended_cc() {
                be_u16(*self as u16)(out)
            } else {
                be_u8(*self as u8)(out)
            }
        }
    }
}
