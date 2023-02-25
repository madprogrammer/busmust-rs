use std::ffi::c_void;
use std::os::raw::c_char;
use bitfield_struct::bitfield;

/// Size (in bytes) of an error description string.
pub const BM_ERROR_DESC_MAX_SIZE: usize = 255;

/// Size (in bytes) of BM Data header, which contains type, routing, length and timestamp.
pub const BM_DATA_HEADER_SIZE: usize = 8;

/// Size (in bytes) of BM Data payload, which contains a concrete message in `CAN-FD` | `LIN` | `FLEXRAY` | `...` type.
pub const BM_DATA_PAYLOAD_MAX_SIZE: usize = 72;

/// Size (in bytes) of BM Data, which contains a header and payload.
pub const BM_DATA_MAX_SIZE: usize = BM_DATA_HEADER_SIZE + BM_DATA_PAYLOAD_MAX_SIZE;

/// Busmust library log level, see [super::api::BM_SetLogLevel] for details.
#[repr(u32)]
pub enum BMLogLevel {
    /// Show nothing on debug console
    None = 0,
    /// Show only ERR level messages on debug console, note this is the default level for release versions
    Error = 1,
    /// Show ERR and WRN level messages on debug console
    Warning = 2,
    /// Show ERR|WRN|INF level messages on debug console
    Info = 3,
    /// Show all messages on debug console, including debug messages, note this is NOT available for release versions
    Debug = 4
}

bitflags!{
    /// Busmust Device capability flags, retrieved when enumerating devices using [super::api::BM_Enumerate].
    #[repr(transparent)]
    pub struct BMCapability: u16 {
        /// No capability
        const NONE = 0x0000;
        /// The device is capable of handling LIN messages
        const LIN = 0x0001;
        /// The device is capable of handling CAN messages
        const CAN = 0x0002;
        /// The device is capable of handling CAN-FD (and CAN) messages
        const CAN_FD = 0x0004;
        /// The device is capable of handling FlexRay messages
        const FLEXRAY = 0x0008;
        /// The device is capable of handling ModBus messages
        const MODBUS = 0x0010;
        /// The device is capable of handling Ethernet messages
        const ETHERNET = 0x0020;
        /// Typically used for masking the CAP fields when programming
        const ALL = 0xFFFF;
    }
}

/// Busmust data type flags, must be given in [BMData].
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum BMDataType {
    /// Unknown data type
    Unknown = 0,
    /// LIN message data type
    Lin = 1,
    /// CAN or CAN-FD message data type (check FDF flag further)
    Can = 2,
    /// FlexRay message data type
    FlexRay = 3,
    /// MODBUS message data type
    ModBus = 4,
    /// Ethernet message data type
    Ethernet = 5,
    /// ACK from bus, which indicates `TXCMPLT` event if this is [BMDataType::Can]
    Ack = 8
}


/// Busmust device & operation status, most APIs would return a status code to indicate the result of an operation
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum BMStatus {
    /// SUCCESS: No error occurred
    Ok = 0x00000,
    /// Low level Transmit buffer is full
    XmtFull = 0x00001,
    /// Bus overrun (the device cannot keep up with the high bus throughput)
    Overrun = 0x00002,
    /// CAN Bus communication is light, see ISO11898 for details
    BusLight = 0x00004,
    /// CAN Bus communication is in warning state, see ISO11898 for details
    BusWarning = 0x00008,
    /// CAN node is in passive state, see ISO11898 for details
    BusPassive = 0x40000,
    /// CAN node failed to transmit message within specified time, the node might be in PASSIVE or BUS-OFF state
    BusTimeout = 0x80000,
    /// CAN bus is in BUS-OFF state, see ISO11898 for details
    BusOff = 0x00010,
    /// CAN bus error occurred
    AnyBusError = 0x00008 | 0x00004 | 0x00010 | 0x40000,
    /// Receive buffer is empty, this might NOT be an error if you use BM API in polling mode
    ReceiveBufferEmpty = 0x00020,
    /// BM API internal queue overrun
    QueueOverrun = 0x00040,
    /// High level Transmit queue is full
    TransmitQueueFull = 0x00080,
    /// Reserved
    RegTest = 0x00100,
    /// Reserved
    NoDriver = 0x00200,
    /// Hardware is in use (opened by another application)
    HardwareInUse = 0x00400,
    /// Reserved
    NetInUse = 0x00800,
    /// Hardware error or invalid hardware handle
    HardwareError = 0x01400,
    /// Invalid bus
    InvalidBus = 0x01800,
    /// Invalid client
    InvalidClient = 0x01C00,
    /// Out of resources
    OutOfResources = 0x02000,
    /// Invalid parameter type in API call
    InvalidParameterType = 0x04000,
    /// Invalid parameter value in API call
    InvalidParameterValue = 0x08000,
    /// Unknown error
    Unknown = 0x10000,
    /// Invalid data received/transmitted
    InvalidData = 0x20000,
    /// Reserved
    Caution = 0x2000000,
    /// The device/library is not initialized
    NotInitialized = 0x4000000,
    /// Invalid operation
    InvalidOperation = 0x8000000
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
/// CAN mode IDs, used by [super::api::BM_SetCanMode] to change the operation mode of CAN device.
pub enum BMCanMode {
    /// The device is running normally (with the capability to handle CAN and CAN-FD messages
    Normal = 0x00,
    /// The device is logically disconnected from CAN bus
    BufOff = 0x01,
    /// The device is looping back messages internally without impacting the physical CAN bus
    InternalLoopback = 0x02,
    /// The device is receiving messages without impacting the physical CAN bus (do not send ACKs to the bus)
    ListenOnly = 0x03,
    /// The device is under configuration and temporarily disconnected from CAN bus, For Internal usage only
    Configuration = 0x04,
    /// The device is looping back messages externally, all transmitted messages are echoed by the device itself
    ExternalLoopback = 0x05,
    /// The device is running normally (with the capability to handle only classical CAN2.0 messages
    Classic = 0x06,
    /// Reserved
    Restricted = 0x07
}

/// Terminal resistor values, used by [super::api::BM_SetTerminalRegister] to change the terminal resistor of CAN device.
#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum BMTerminalResistor {
    /// Reserved, currently unsupported
    Auto = 0,
    /// Currently unsupported
    Enabled60 = 60,
    /// 120Ohm
    Enabled120 = 120,
    /// Disable terminal resistor
    Disabled = 0xFFFF
}

/// CAN Message type flags, used in [BMCanMessage]
#[repr(u8)]
pub enum BMMessageFlags {
    /// Normal CAN message
    Normal = 0,
    /// Extended CAN message
    Extended = 1,
    /// Remote CAN message
    Remote = 2,
    /// CAN-FD bitrate switching is enabled
    BitRateSwitching = 4,
    /// CAN-FD message
    CanFD = 8,
    /// Reserved for gateways
    Esi = 16
}

/// CAN RX filter type IDs, used in [BMRxFilter]
#[repr(u32)]
pub enum BMRxFilterType {
    /// Invalid (unused) RX filter entry
    Invalid = 0,
    /// Basic RX filter, traditional acceptance filter based on message ID mask
    Basic = 1,
    /// Busmust advanced RX filter, check both message ID and message payload
    Advanced = 2,
    /// Busmust E2E RX filter, accept only messages that passed E2E checking
    E2EPass = 3,
    /// Busmust E2E RX filter, accept only messages that failed E2E checking (for debugging purpose)
    E2EFail = 4
}

/// CAN TX task type IDs, used in [BMTxTask].
#[repr(u32)]
pub enum BMTxTaskType {
    /// Invalid (unused) TX task entry
    Invalid = 0,
    /// Basic TX task, send fixed ID and fixed payload
    Fixed = 1,
    /// Self-increment Data TX task
    IncData = 2,
    /// Self-increment ID TX task
    IncId = 3,
    /// Random Data TX task
    RandomData = 4,
    /// Random ID TX task
    RandomId = 5
}

/// CAN runtime statistics item IDs, used in [super::api::BM_GetStatus].
#[repr(u32)]
pub enum BMStatType {
    /// Invalid statistics item
    None = 0,
    /// Number of TX messages
    TxMessage = 1,
    /// Number of RX messages
    RxMessage = 2,
    /// Number of TX bytes
    TxByte = 3,
    /// Number of RX bytes
    RxByte = 4,
    /// Number of TX errors
    TxError = 5,
    /// Number of RX errors
    RxError = 6
}

/// ISO-TP operation mode.
#[repr(u32)]
pub enum BMIsotpMode {
    /// Default mode: normal (non-extended-addressing) UDS client(tester)
    NormalTester = 0,
    /// Normal (non-extended-addressing) UDS server(ECU)
    NormalEcu = 1,
    /// Currently unsupported: extended-addressing UDS client(tester)
    ExtendedTester = 2,
    /// Currently unsupported: extended-addressing UDS server(ECU)
    ExtendedEcu = 3
}

/// Busmust data header, each BMData contains a header which indicates payload information.
#[bitfield(u16)]
#[derive(Default)]
pub struct BMDataHeader {
    /// Data type, see BMDataType for details.
    #[bits(4)]
    pub kind: u8,
    /// Reserved flags, keep 0
    #[bits(4)]
    pub flags: u8,
    /// Destination channel ID, starting from zero, used by TX data to indicate the hardware about the target port.
    #[bits(4)]
    pub dchn: u8,
    /// Source channel ID, starting from zero, used by RX data to indicate the application about the source port.
    #[bits(4)]
    pub schn: u8,
}

/// Busmust data, abstract structure which holds concrete payload messages of various types (i.e. CAN messages).
#[repr(C)]
pub struct BMData {
    /// Data header, see [BMDataHeader] for details.
    pub header: BMDataHeader,
    /// Length in bytes of the payload only (header excluded).
    pub length: u16,
    /// 32-bit device local high precision timestamp in microseconds.
    pub timestamp: u32,
    /// Buffer holding concrete message payload (i.e. a CAN message in [BMCanMessage] format).
    pub payload: [u8; BM_DATA_PAYLOAD_MAX_SIZE],
}

/// Busmust CAN Message ID.
/// You could also use an `u32`, but please take care of memory alignments.
#[bitfield(u32)]
pub struct BMMessageId {
    /// Standard ID
    #[bits(11)]
    pub sid: u16,
    /// Extended ID
    #[bits(18)]
    pub eid: u32,
    /// Reserved
    #[bits(1)]
    pub reserved1: bool,
    /// Reserved
    #[bits(2)]
    pub reserved2: u8,
}

/// Busmust TX CAN Message control fields.
/// The first few fields (until FDF) are bit compatible with [BMRxMessageCtrl].
#[bitfield(u32)]
pub struct BMTxMessageCtrl {
    /// CAN message DLC(0-F), note this is not the message length
    #[bits(4)]
    pub dlc: u8,
    /// This message is an extended CAN message
    #[bits(1)]
    pub ide: bool,
    /// This message is a remote CAN message
    #[bits(1)]
    pub rtr: bool,
    /// This message requires CAN-FD bitrate switching
    #[bits(1)]
    pub brs: bool,
    /// This message is a CAN-FD CAN message
    #[bits(1)]
    pub fdf: bool,
    /// Reserved for gateways
    #[bits(1)]
    pub esi: bool,
    /// Reserved for hardware sync
    #[bits(23)]
    pub seq: u32,
}

/// Busmust RX CAN Message control fields.
/// The first few fields (until FDF) are bit compatible with [BMTxMessageCtrl].
#[bitfield(u32)]
pub struct BMRxMessageCtrl {
    /// CAN message DLC(0-F), note this is not the message length
    #[bits(4)]
    pub dlc: u8,
    /// This message is an extended CAN message
    #[bits(1)]
    pub ide: bool,
    /// This message is a remote CAN message
    #[bits(1)]
    pub rtr: bool,
    /// This message requires CAN-FD bitrate switching
    #[bits(1)]
    pub brs: bool,
    /// This message is a CAN-FD CAN message
    #[bits(1)]
    pub fdf: bool,
    /// Reserved for gateways
    #[bits(1)]
    pub esi: bool,
    /// Reserved
    #[bits(2)]
    pub reserved1: u8,
    /// By which RX filter the message was accepted
    #[bits(5)]
    pub rx_filter: u8,
    /// Reserved
    #[bits(16)]
    pub reserved2: u16,
}

/// Busmust CAN Message control fields.
#[repr(C)]
#[derive(Copy, Clone)]
pub union BMMessageCtrl {
    /// TX control
    pub tx: BMTxMessageCtrl,
    /// RX control
    pub rx: BMRxMessageCtrl
}

/// Busmust CAN Message concrete type, usually used as payload of [BMData].
/// The total length of this structure is 72B, it supports both classic and FD CAN messages.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BMCanMessage {
    /// CAN message ID, see [BMMessageId] for details.
    pub mid: BMMessageId,
    /// CAN message control fields, whether TX or RX is taken depends on the message direction.
    pub ctrl: BMMessageCtrl,
    /// CAN message payload
    pub payload: [u8; 64]
}

/// Channel information, created when enumerating devices by [super::api::BM_Enumerate] and used when opening device by [super::api::BM_OpenEx]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BMChannelInfo {
    /// Device full name, for display purpose
    pub name: [c_char; 64],
    /// Device SN
    pub sn: [u8; 16],
    /// Device UID
    pub uid: [u8; 12],
    /// Device Firmware Version
    pub version: [u8; 4],
    /// Device VID
    pub vid: u16,
    /// Device PID
    pub pid: u16,
    /// Port ID (0-7) of the device, note a multi-port device is enumerated as multiple dedicated [BMChannelInfo] entries
    pub port: u16,
    /// Device Capability flags, see [BMCapability] for details.
    pub cap: BMCapability,
    /// Reserved
    pub reserved: [u8; 4]
}

/// CAN channel status detailed information, retrieved by calling [super::api::BM_GetStatus], see ISO11898 for details.
#[repr(C)]
#[derive(Debug)]
pub struct BMCanStatusInfo {
    /// The CAN channel is in BUS-OFF state
    pub tx_bus_off: u8,
    /// Reserved
    reserved: u8,
    /// The CAN channel is in TX bus passive state
    pub tx_bus_passive: u8,
    /// The CAN channel is in RX bus passive state
    pub rx_bus_passive: u8,
    /// The CAN channel is in TX warn state
    pub tx_warn: u8,
    /// The CAN channel is in RX warn state
    pub rx_warn: u8,
    /// TX Bus Error counter
    pub tx_errors: u8,
    /// RX Bus Error counter
    pub rx_errors: u8
}

/// CAN channel bitrate configuration, used by [super::api::BM_SetBitrate]
#[repr(C)]
pub struct BMBitrate {
    /// Nominal bitrate in kbps, default as 500, note this is the only valid bitrate in CAN CLASSIC mode.
    pub n_bitrate: u16,
    /// Data bitrate in kbps, default as 500, note this is ignored in CAN CLASSIC mode.
    pub d_bitrate: u16,
    /// Nominal sample position (percentage, 0-100, default as 75
    pub n_sample_pos: u8,
    /// Data sample position (percentage, 0-100, default as 75
    pub d_sample_pos: u8,
    // Setting any of the fields below would override the n_bitrate configuration
    /// CAN controller clock in Mhz, default as 0
    pub clock_freq: u8,
    /// Reserved
    pub reserved: u8,
    /// Nominal BTR0 register value, note this value is calculated using clock_freq, which might not be 16MHz
    pub n_btr0: u8,
    /// Nominal BTR1 register value, note this value is calculated using clock_freq, which might not be 16MHz
    pub n_btr1: u8,
    /// Data BTR0 register value, note this value is calculated using clock_freq, which might not be 16MHz
    pub d_btr0: u8,
    /// Data BTR1 register value, note this value is calculated using clock_freq, which might not be 16MHz
    pub d_btr1: u8
}

/// CAN channel RX filter item structure, used by [super::api::BM_SetRxFilters]
/// The filter support masking ID, flags and payload according to its type,
/// in order for a message to be accepted, all the fields are masked using AND logic:
/// `(flags & filter.flags_mask == filter.flags_value) AND (ID & filter.id_mask == filter.id_value) AND (payload & filter.payload_mask == filter.payload_value)`
#[repr(C)]
pub struct BMRxFilter {
    /// Type ID of the RX filter, see [BMRxFilterType] for details.
    pub kind: u8,
    /// Reserved
    pub unused: u8,
    /// CAN message control Flags masks, see [BMMessageFlags] for details.
    pub flags_mask: u8,
    /// CAN message control Flags values, see [BMMessageFlags] for details.
    pub flags_value: u8,
    /// Reserved
    pub reserved: [u8; 4],
    pub id_mask: u32,
    /// CAN message ID values, see [BMMessageId] for details.
    pub id_value: u32,
    /// CAN message payload masks, for CAN-FD messages, only the first 8 bytes are checked.
    pub payload_mask: [u8; 8],
    /// CAN message payload values, for CAN-FD messages, only the first 8 bytes are checked.
    pub payload_value: [u8; 8]

}

#[repr(C)]
pub struct BMTxTaskIncDataPattern {
    /// Start bit of data increment, currently only 8-bit aligned value is accepted
    pub start_bit: u16,
    /// Number of bits of data increment, currently only 32 is accepted
    pub nbits: u8,
    /// Number of bits of data increment, currently only 32 is accepted
    pub format: u8,
    /// Minimum value of the Increment range
    pub min: u32,
    /// Maximum value of the Increment range
    pub max: u32,
    /// Step of the Increment range
    pub step: u32
}

#[repr(C)]
pub struct BMTxTaskIncIdPattern {
    /// Minimum value of the Increment range
    pub min: u32,
    /// Maximum value of the Increment range
    pub max: u32,
    /// Step of the Increment range
    pub step: u32
}

#[repr(C)]
pub struct BMTxTaskRndDataPattern {
    /// Start bit of data Random, currently only 8-bit aligned value is accepted
    pub start_bit: u16,
    /// Number of bits of data Random, currently only 32 is accepted
    pub nbits: u8,
    /// 0x80=Intel, 0x00=Motorola
    pub format: u8,
    /// Minimum value of the Random range
    pub min: u32,
    /// Maximum value of the Random range
    pub max: u32,
    /// Seed of the Random range
    pub seed: u32
}

#[repr(C)]
pub struct BMTxTaskRndIdPattern {
    /// Minimum value of the Increment range
    pub min: u32,
    /// Maximum value of the Increment range
    pub max: u32,
    /// Seed of the Random range
    pub seed: u32
}

/// CAN channel TX task item structure, used by [super::api::BM_SetTxTasks]
/// Once the CAN device is armed with TX tasks, it will try to parse the TX task and send CAN messages automatically.
/// The difference with a software triggered CAN message in BusMaster is that hardware triggered CAN messages are
/// more precise in time and could reach a higher throughput.
#[repr(C)]
pub struct BMTxTask {
    /// Type ID of the TX task, see [BMTxTaskType] for details.
    pub kind: u8,
    /// Reserved
    pub unused: u8,
    /// CAN message control Flags, see [BMMessageFlags] for details.
    pub flags: u8,
    /// Length of payload in bytes (not DLC)
    pub length: u8,
    /// Index of E2E (in E2E table, currently unsupported
    pub e2e: u8,
    /// Reserved
    pub reserved: u8,
    /// ms delay between rounds
    pub cycle: u16,
    /// num of cycles
    pub n_rounds: u16,
    /// messages per round
    pub n_messages: u16,
    /// CAN message arbitration ID, see [BMMessageId] for details.
    pub id: u32,
    /// TX task pattern data
    pub pattern: u32,
    //TODO
    /// Default payload data, note this is also the template payload of the unchanged part in a volatile TX task
    pub payload: [u8; 64]
}

/// ISO-TP status report, used by ISO-TP operation callback function.
#[repr(C)]
pub struct BMIsoTPStatus {
    /// Currently always 0x01
    pub version: u8,
    /// Current flow control status, 0=continue, 1=wait, 2=overflow, ff=timeout,
    pub flow_control: u8,
    /// Current flow control status, i.e. 30 00 00
    pub st_min: u8,
    /// Current block size
    pub block_size: u8,
    /// Number of transferred bytes by now.
    pub transferred_bytes: u32,
    /// Number of total bytes indicated by ISO-TP FF or SF.
    pub total_bytes: u32,
    /// Current timestamp reported by device.
    pub timestamp: u32,
    /// Reserved for future.
    pub reserved: [u32; 4]
}

/// Callback type for ISO-TP status callback
type BMIsotpCallbackHandle = extern fn(status: *const BMIsoTPStatus, arg: *const c_void);

#[repr(C)]
pub struct BMIsotpTimeoutConfig {
    /// `A` timeout in milliseconds: `=N_As` if writing as tester or reading as ECU, otherwise `=N_Ar`
    pub a: u16,
    /// `B` timeout in milliseconds: `=N_Bs` if writing as tester or reading as ECU, otherwise `=N_Br`
    pub b: u16,
    /// `C` timeout in milliseconds: `=N_Cs` if writing as tester or reading as ECU, otherwise `=N_Cr`
    pub c: u16
}


#[repr(C)]
pub struct BMIsotpFlowControlConfig {
    /// STmin raw value (0x00-0x7F or 0xF1-0xF9) if Busmust device is acting as UDS server
    pub st_min: u8,
    /// Block size if CAN card is acting as UDS server, 0 means no further FC is needed
    pub block_size: u8,
    /// Flow control frame length in bytes
    pub fc_frame_length: u8,
    /// Reserved
    pub reserved: u8
}

#[repr(C)]
/// ISOTP Protocol (See ISO15765-2 for details) configuration.
pub struct BMIsotpConfig {
    /// Currently must be set to 0x01
    pub version: u8,
    /// Currently only 0 is supported: normal (non-extended-addressing) UDS client(tester)
    pub mode: u8,
    /// Tester(UDS Client) Timeout configuration
    pub tester_timeout: BMIsotpTimeoutConfig,
    /// ECU(UDS Server) Timeout configuration
    pub ecu_timeout: BMIsotpTimeoutConfig,
    /// Current flow control status, i.e. 30 00 00
    pub flow_control: BMIsotpFlowControlConfig,
    /// UDS Address in Extended Addressing mode
    pub extended_address: u8,
    /// Enable padding for unused payload bytes
    pub padding_enabled: u8,
    /// Padding byte value (i.e. 0xCC) for unused payload bytes
    pub padding_value: u8,
    /// Enable long PDU (>4095), note if CAN message DLC>8, long PDU is enabled by default
    pub long_pdu_enabled: u8,
    /// Reserved for future
    pub padding: [u8; 2],
    /// Callback function when any progress is made, used typically by GUI to show progress bar
    pub callback: BMIsotpCallbackHandle,
    /// Callback user arg when any progress is made, used typically by GUI to show progress bar
    pub callback_user_arg: *const c_void,
    /// All tester messages will be formatted/checked using this template, configure CAN message ID and IDE/FDF flags here
    pub tester_data_template: BMData,
    /// All ECU messages will be formatted/checked using this template, configure CAN message ID and IDE/FDF flags here
    pub ecu_data_template: BMData
}
