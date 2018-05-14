use std::os::raw::{c_char, c_int};

pub type BOOL = c_int;

/* ------------------------------------------------------------ */
/*                Miscellaneous Declarations                    */
/* ------------------------------------------------------------ */

pub const MAX_PATH: usize = 260; // this is the current windows definition

/* ------------------------------------------------------------ */
/*                General Type Declarations                     */
/* ------------------------------------------------------------ */

/* These symbols define the maximum allowed length for various
** strings used by the interfaces.
*/
pub const cchAliasMax: usize = 16; //max length of device table alias string
pub const cchUsrNameMax: usize = 16; //max length of user settable device name
pub const cchProdNameMax: usize = 28; //max length of product name string
pub const cchSnMax: usize = 15; //length of a serial number string
pub const cchVersionMax: usize = 256; //max length returned for DLL version string
pub const cchDvcNameMax: usize = 64; //size of name field in DVC structure
pub const cchDtpStringMax: usize = 16; //maximum length of DTP name string
pub const cchErcMax: usize = 48; //maximum length of error code symbolic name
pub const cchErcMsgMax: usize = 128; //maximum length of error message descriptive string

/* The device management capabilities value indicates which device
** management function sets are supported by the device. Device
** management function sets apply to a device as a whole. For example,
** the mgtcapPower capability indicates that the device supports the
** power on/off capability.
*/
pub type MGTCAP = u32; // management capabilities

/* The device interface capabilties value indicates which interface types
** are supported by the device or being requested by the application.
*/
pub type DCAP = u32; //capabilities bitfield

pub const dcapJtag: DCAP = 0x00000001; //this symbol is deprecated
pub const dcapJtg: DCAP = 0x00000001;
pub const dcapPio: DCAP = 0x00000002;
pub const dcapEpp: DCAP = 0x00000004;
pub const dcapStm: DCAP = 0x00000008;
pub const dcapSpi: DCAP = 0x00000010;
pub const dcapTwi: DCAP = 0x00000020;
pub const dcapAci: DCAP = 0x00000040;
pub const dcapAio: DCAP = 0x00000080;
pub const dcapEmc: DCAP = 0x00000100;
pub const dcapDci: DCAP = 0x00000200;
pub const dcapGio: DCAP = 0x00000400;
pub const dcapPti: DCAP = 0x00000800;

pub const dcapAll: DCAP = 0xFFFFFFFF;

/* The port properties values are used by each protocol type to
** indicate details about the features supported by each individual
** port. The type is declared here. The properties values are
** defined in the protocol specific header files.
*/
pub type DPRP = u32;

/* Device type indicates which physical transport and protocol are used to
** access the device. The lower 16 bits are interpreted as a bitfield that
** is used to specify the type of transport used by the device. The upper
** 16 bits are interpreted as the protocol used to communicate with a
** device of the specified transport type. Please note that specification
** of the protocol is optional and if no protocol is specified then
** communication with all devices of a particular transport type will be
** attempted.
*/
pub type DTP = u32;

pub const dtpUSB: DTP = 0x00000001;
pub const dtpEthernet: DTP = 0x00000002;
pub const dtpParallel: DTP = 0x00000004;
pub const dtpSerial: DTP = 0x00000008;

pub const dtpNone: DTP = 0x00000000;
pub const dtpAll: DTP = 0xFFFFFFFF;
pub const dtpNil: DTP = 0;

pub type TPT = u16;

pub const tptUSB: TPT = 0x0001;
pub const tptEthernet: TPT = 0x0002;
pub const tptParallel: TPT = 0x0004;
pub const tptSerial: TPT = 0x0008;

pub const tptNone: TPT = 0x0000;
pub const tptAll: TPT = 0xFFFF;
pub const tptNil: TPT = 0x0000;

pub type PTC = u16;

pub const ptcProtocol1: PTC = 0x0001;
pub const ptcProtocol2: PTC = 0x0002;
pub const ptcProtocol3: PTC = 0x0004;
pub const ptcProtocol4: PTC = 0x0008;

pub const ptcProtocol5: PTC = 0x0010;
pub const ptcProtocol6: PTC = 0x0020;
pub const ptcProtocol7: PTC = 0x0040;
pub const ptcProtocol8: PTC = 0x0080;

pub const ptcProtocol9: PTC = 0x0100;
pub const ptcProtocol10: PTC = 0x0200;
pub const ptcProtocol11: PTC = 0x0400;
pub const ptcProtocol12: PTC = 0x0800;

pub const ptcProtocol13: PTC = 0x1000;
pub const ptcProtocol14: PTC = 0x2000;
pub const ptcProtocol15: PTC = 0x4000;
pub const ptcProtocol16: PTC = 0x8000;

pub const ptcAll: PTC = 0x0000;
pub const ptcNil: PTC = 0x0000;

#[inline]
pub unsafe fn TptFromDtp(dtp: DTP) -> TPT {
    (dtp & 0xFFFF) as TPT
}

#[inline]
pub unsafe fn PtcFromDtp(dtp: DTP) -> PTC {
    ((dtp >> 16) & 0xFFFF) as PTC
}

#[inline]
pub unsafe fn DtpFromTptPtc(tpt: TPT, ptc: PTC) -> DTP {
    tpt as DTP | (ptc as DTP) << 16
}

/* Device interface handle.
*/
pub type HIF = u32;
pub const hifInvalid: HIF = 0;

/* These values are used to report various attributes of a device.
*/
pub type PDID = u32; // device product id
pub type FWTYPE = u16;
pub type FWVER = u16; // device firmware version number
pub type FWID = u8; // device firmware identifier

#[inline]
pub unsafe fn ProductFromPdid(pdid: PDID) -> c_int {
    ((pdid >> 20) & 0xFFF) as c_int
}

#[inline]
pub unsafe fn VariantFromPdid(pdid: PDID) -> c_int {
    ((pdid >> 8) & 0xFFF) as c_int
}

#[inline]
pub unsafe fn FwidFromPdid(pdid: PDID) -> FWID {
    (pdid & 0xFF) as FWID
}

/* These values are used to retrieve or set various information about
** a device.
*/
pub type DINFO = u32;

// public
pub const dinfoNone: DINFO = 0;
pub const dinfoAlias: DINFO = 1;
pub const dinfoUsrName: DINFO = 2;
pub const dinfoProdName: DINFO = 3;
pub const dinfoPDID: DINFO = 4;
pub const dinfoSN: DINFO = 5;
pub const dinfoIP: DINFO = 6;
pub const dinfoMAC: DINFO = 7; //Ethernet MAC and SN are the same
pub const dinfoDCAP: DINFO = 9;
pub const dinfoSerParam: DINFO = 10;
pub const dinfoParAddr: DINFO = 11;
pub const dinfoUsbPath: DINFO = 12;
pub const dinfoProdID: DINFO = 13; // the ProductID from PDID
pub const dinfoOpenCount: DINFO = 14; // how many times a device is opened
pub const dinfoFWVER: DINFO = 15;

/* Error codes
*/
pub type ERC = c_int;

pub const ercNoErc: ERC = 0; //  No error occurred

// The following error codes can be directly mapped to the device error codes.
pub const ercNotSupported: ERC = 1; //  Capability or function not supported by the device
pub const ercTransferCancelled: ERC = 2; //  The transfer was cancelled or timeout occured
pub const ercCapabilityConflict: ERC = 3; //  Tried to enable capabilities that use shared resources, check device datasheet
pub const ercCapabilityNotEnabled: ERC = 4; //  The protocol is not enabled
pub const ercEppAddressTimeout: ERC = 5; //  EPP Address strobe timeout
pub const ercEppDataTimeout: ERC = 6; //  EPP Data strobe timeout
pub const ercDataSndLess: ERC = 7; //  Data send failed or peripheral did not received all the sent data
pub const ercDataRcvLess: ERC = 8; //  Data receive failed or peripheral sent less data
pub const ercDataRcvMore: ERC = 9; //  Peripheral sent more data
pub const ercDataSndLessRcvLess: ERC = 10; //  Two errors: ercDataSndLess and ercDataRcvLess
pub const ercDataSndLessRcvMore: ERC = 11; //  Two errors: ercDataSndLess and ercDataSndFailRcvMore
pub const ercInvalidPort: ERC = 12; //  Attempt to enable port when another port is already enabled
pub const ercBadParameter: ERC = 13; //  Command parameter out of range

// ACI error codes, directly mapped to device error codes.
pub const ercAciFifoFull: ERC = 0x20; //  Transmit FIFO overflow

// TWI error codes, directly mapped to device error codes.
pub const ercTwiBadBatchCmd: ERC = 0x20; //  Bad command in TWI batch buffer
pub const ercTwiBusBusy: ERC = 0x21; //  Timed out waiting for TWI bus
pub const ercTwiAdrNak: ERC = 0x22; //  TWI address not ack'd
pub const ercTwiDataNak: ERC = 0x23; //  TWI data not ack'd
pub const ercTwiSmbPecError: ERC = 0x24; //  Packet error when using packet error checking

// Most likely the user did something wrong.
pub const ercAlreadyOpened: ERC = 1024; //  Device already opened
pub const ercInvalidHif: ERC = 1025; //  Invalid interface handle provided, fist call DmgrOpen(Ex)
pub const ercInvalidParameter: ERC = 1026; //  Invalid parameter sent in API call
pub const ercTransferPending: ERC = 1031; //  The last API called in overlapped mode was not finished. Use DmgrGetTransStat or DmgrCancelTrans
pub const ercApiLockTimeout: ERC = 1032; //  API waiting on pending API timed out
pub const ercPortConflict: ERC = 1033; //  Attempt to enable port when another port is already enabled

// Not the user's fault.
pub const ercConnectionFailed: ERC = 3072; //  Unknown fail of connection
pub const ercControlTransferFailed: ERC = 3075; //  Control transfer failed
pub const ercCmdSendFailed: ERC = 3076; //  Command sending failed
pub const ercStsReceiveFailed: ERC = 3077; //  Status receiving failed
pub const ercInsufficientResources: ERC = 3078; //  Memory allocation failed, insufficient system resources
pub const ercInvalidTFP: ERC = 3079; //  Internal protocol error, DVT rejected the transfer strcuture sent by public API
pub const ercInternalError: ERC = 3080; //  Internal error
pub const ercTooManyOpenedDevices: ERC = 3081; //  Internal error
pub const ercConfigFileError: ERC = 3082; //  Processing of configuration file failed
pub const ercDeviceNotConnected: ERC = 3083; //  Device not connected

pub const ercEnumNotFree: ERC = 3084; //  Device Enumeration failed because another enumeration is still running.
pub const ercEnumFreeFail: ERC = 3085; //  Device Enumeration list could not be freed

pub const ercInvalidDevice: ERC = 3086; //  OEM ID check failed

pub const ercDeviceBusy: ERC = 3087; //  The device is currently claimed by another process.

pub const ercCorruptInstallation: ERC = 3088; //  One or more critical file is missing from the system.

pub const ercDabsInitFailed: ERC = 3089; //  Initialization of the DABS library failed
pub const ercDpcommInitFailed: ERC = 3090; //  Initialization of the DPCOMM library failed

//ENUM errors

//DVTBL errors

/* ------------------------------------------------------------ */
/*                  Data Structure Declarations                 */
/* ------------------------------------------------------------ */

#[repr(C)]
pub struct DVC {
    pub szName: [c_char; cchDvcNameMax],
    //in dvctable:  Alias
    //not in dvctable:  user assigned name in device
    //not in dvctable, no user defined name:  device type with identifier
    pub szConn: [c_char; MAX_PATH + 1],
    //in dvctable:  connection string in dvctable
    //not in dvctable:  USB:   PATHNAME
    //                  Eth:    IP:192.168.1.1
    //                  Ser:    COM1:9600,N,8,1
    //                  EPP:    EPP:0x378
    pub dtp: DTP,
}
