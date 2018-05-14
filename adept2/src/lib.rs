extern crate adept2_sys as ffi;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate failure;

mod call;
mod util;

pub mod dmgr;
pub mod depp;
pub mod djtag;

use std::fmt;

bitflags!{
    pub struct DeviceCaps: ffi::DCAP {
        const JTAG = 0x00000001;
        const PIO = 0x00000002;
        const EPP = 0x00000004;
        const STM = 0x00000008;
        const SPI = 0x00000010;
        const TWI = 0x00000020;
        const ACI = 0x00000040;
        const AIO = 0x00000080;
        const EMC = 0x00000100;
        const DCI = 0x00000200;
        const GIO = 0x00000400;
        const PTI = 0x00000800;
        const ALL = 0xFFFFFFFF;
    }
}

bitflags! {
    /// Device type indicates which physical transport and protocol are used to
    /// access the device. The lower 16 bits are interpreted as a bitfield that
    /// is used to specify the type of transport used by the device. The upper
    /// 16 bits are interpreted as the protocol used to communicate with a
    /// device of the specified transport type. Please note that specification
    /// of the protocol is optional and if no protocol is specified then
    /// communication with all devices of a particular transport type will be
    /// attempted.
    pub struct DeviceType: ffi::DTP {
        const USB = 0x00000001;
        const ETHERNET = 0x00000002;
        const PARALLEL = 0x00000004;
        const SERIAL = 0x00000008;

        const ALL = 0xFFFFFFFF;
    }
}

impl DeviceType {
    pub fn from_parts(t: Transport, p: Protocol) -> DeviceType {
        unsafe { DeviceType::from_bits_truncate(ffi::DtpFromTptPtc(t.bits(), p.bits())) }
    }
}

bitflags! {
    pub struct Transport : ffi::TPT {
        const USB = 0x0001;
        const ETHERNET = 0x0002;
        const PARALLEL = 0x0004;
        const SERIAL = 0x0008;

        const ALL = 0xFFFF;
    }
}

impl From<DeviceType> for Transport {
    fn from(d: DeviceType) -> Transport {
        unsafe { Transport::from_bits_truncate(ffi::TptFromDtp(d.bits())) }
    }
}

bitflags! {
    pub struct Protocol : ffi::PTC {
        const PROTOCOL1 = 0x0001;
        const PROTOCOL2 = 0x0002;
        const PROTOCOL3 = 0x0004;
        const PROTOCOL4 = 0x0008;

        const PROTOCOL5 = 0x0010;
        const PROTOCOL6 = 0x0020;
        const PROTOCOL7 = 0x0040;
        const PROTOCOL8 = 0x0080;

        const PROTOCOL9 = 0x0100;
        const PROTOCOL10 = 0x0200;
        const PROTOCOL11 = 0x0400;
        const PROTOCOL12 = 0x0800;

        const PROTOCOL13 = 0x1000;
        const PROTOCOL14 = 0x2000;
        const PROTOCOL15 = 0x4000;
        const PROTOCOL16 = 0x8000;

        const ALL = 0xFFFF;
    }
}

impl From<DeviceType> for Protocol {
    fn from(d: DeviceType) -> Protocol {
        unsafe { Protocol::from_bits_truncate(ffi::PtcFromDtp(d.bits())) }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ProductId(ffi::PDID);

impl ProductId {
    pub fn product(&self) -> usize {
        unsafe { ffi::ProductFromPdid(self.0) as usize }
    }

    pub fn variant(&self) -> usize {
        unsafe { ffi::VariantFromPdid(self.0) as usize }
    }

    pub fn firmware_id(&self) -> usize {
        unsafe { ffi::FwidFromPdid(self.0) as usize }
    }
}

#[derive(Debug, Fail)]
pub struct Error(ffi::ERC);

pub type Result<T> = std::result::Result<T, failure::Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (symbolic, desc) = dmgr::desc_from_error(self);
        write!(
            f,
            "An error occurred with error code {}/ ({})",
            symbolic, desc
        )
    }
}
