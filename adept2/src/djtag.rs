use super::{DeviceCaps, Error, Result};
use call::cvt_r;
use dmgr::{Device, DeviceHandle};
use ffi;
use std::{mem, ptr};
use std::os::raw::c_char;
use std::time::Duration;
use util::StringExt;

const INVALID_PARAMETER: Error = Error(ffi::ercInvalidParameter);

bitflags! {
    pub struct PortProperties: ffi::DPRP {
        /// port supports set speed call
        const SET_SPEED = 0x00000001;
        /// device fully implements DjtgSetTmsTdiTck
        const SET_PIN_STATE = 0x00000002;
        /// port supports transaction buffering
        const TRANS_BUFFERING = 0x00000004;
        /// port supports DjtgWait
        const WAIT = 0x00000008;
        /// port supports DjtgSetDelayCnt and DjtgGetDelayCnt
        const DELAY_CNT = 0x00000010;
        /// port supports DjtgSetReadyCnt and DjtgGetReadyCnt
        const READY_CNT = 0x00000020;
        /// port supports DjtgEscape
        const ESCAPE = 0x00000040;
        /// port supports the MScan format
        const MSCAN = 0x00000080;
        /// port supports the OScan0 format
        const OSCAN0 = 0x00000100;
        /// port supports the OScan1 format
        const OSCAN1 = 0x00000200;
        /// port supports the OScan2 format
        const OSCAN2 = 0x00000400;
        /// port supports the OScan3 format
        const OSCAN3 = 0x00000800;
        /// port supports the OScan4 format
        const OSCAN4 = 0x00001000;
        /// port supports the OScan5 format
        const OSCAN5 = 0x00002000;
        /// port supports the OScan6 format
        const OSCAN6 = 0x00004000;
        /// port supports the OScan7 format
        const OSCAN7 = 0x00008000;
        /// port supports DjtgCheckPacket
        const CHECK_PACKET = 0x00010000;
        /// port supports DjtgBatch
        const BATCH = 0x00020000;
        /// port supports DjtgSetAuxReset
        const SET_AUX_RESET = 0x00040000;
        /// port supports DjtgSetGpioState and related functions
        const SET_GET_GPIO = 0x00080000;
    }
}

bitflags! {
    pub struct BatchProperties: u32 {
        const WAIT_US = 0x00000001;
        const SET_AUX_RESET = 0x00000002;
        const SET_GET_GPIO = 0x00000004;
    }
}

#[repr(u8)]
pub enum ScanFormat {
    None = 0,
    JScan0 = 1,
    JScan1 = 2,
    JScan2 = 3,
    JScan3 = 4,
    MScan = 5,
    OScan0 = 6,
    OScan1 = 7,
    OScan2 = 8,
    OScan3 = 9,
    OScan4 = 10,
    OScan5 = 11,
    OScan6 = 12,
    OScan7 = 13,
}

pub fn get_version() -> Result<String> {
    unsafe {
        let mut version: [c_char; ffi::cchVersionMax] = mem::zeroed();
        cvt_r(ffi::DjtgGetVersion(version.as_mut_ptr()))?;
        Ok(String::from_slice(&version[..]))
    }
}

pub struct Djtag(DeviceHandle);

impl Djtag {
    pub fn new(device: &Device) -> Result<Djtag> {
        device
            .device_caps()
            .and_then(|caps| {
                if caps.contains(DeviceCaps::JTAG) {
                    device.open()
                } else {
                    Err(Error(ffi::ercNotSupported).into())
                }
            })
            .and_then(|hif| {
                enable(&hif)?;
                Ok(Djtag(hif))
            })
    }

    pub fn port_count(&self) -> Result<usize> {
        let mut count: i32 = 0;
        unsafe {
            cvt_r(ffi::DjtgGetPortCount(self.0.raw(), &mut count))?;
        }
        Ok(count as usize)
    }

    pub fn port_properties(&self, port: i32) -> Result<PortProperties> {
        let mut props: u32 = 0;
        unsafe {
            cvt_r(ffi::DjtgGetPortProperties(self.0.raw(), port, &mut props))?;
        }
        Ok(PortProperties::from_bits_truncate(props))
    }

    pub fn batch_properties(&self, port: i32) -> Result<BatchProperties> {
        let mut props: u32 = 0;
        unsafe {
            cvt_r(ffi::DjtgGetBatchProperties(self.0.raw(), port, &mut props))?;
        }
        Ok(BatchProperties::from_bits_truncate(props))
    }

    pub fn enable_port(&mut self, port: usize) -> Result<()> {
        let port = port as i32;
        unsafe { cvt_r(ffi::DjtgEnableEx(self.0.raw(), port)) }
    }

    // configuration functions
    pub fn speed(&self) -> Result<usize> {
        let mut freq: u32 = 0;
        unsafe {
            cvt_r(ffi::DjtgGetSpeed(self.0.raw(), &mut freq))?;
        }
        Ok(freq as usize)
    }

    pub fn tms_tdi_tdo_tck(&self) -> Result<(bool, bool, bool, bool)> {
        let mut tms: ffi::BOOL = 0;
        let mut tdi: ffi::BOOL = 0;
        let mut tdo: ffi::BOOL = 0;
        let mut tck: ffi::BOOL = 0;
        unsafe {
            cvt_r(ffi::DjtgGetTmsTdiTdoTck(
                self.0.raw(),
                &mut tms,
                &mut tdi,
                &mut tdo,
                &mut tck,
            ))?;
        }
        Ok((tms > 0, tdi > 0, tdo > 0, tck > 0))
    }

    pub fn set_speed(&self, freq: u32) -> Result<u32> {
        let mut actual: u32 = 0;
        unsafe {
            cvt_r(ffi::DjtgSetSpeed(self.0.raw(), freq, &mut actual))?;
        }
        Ok(actual)
    }

    pub fn set_tms_tdi_tck(&self, tms: bool, tdi: bool, tck: bool) -> Result<()> {
        let tms = tms as ffi::BOOL;
        let tdi = tdi as ffi::BOOL;
        let tck = tck as ffi::BOOL;
        unsafe { cvt_r(ffi::DjtgSetTmsTdiTck(self.0.raw(), tms, tdi, tck)) }
    }

    pub fn set_aux_reset(&self, reset: bool, enable_output: bool) -> Result<()> {
        let reset = reset as ffi::BOOL;
        let enable_output = enable_output as ffi::BOOL;
        unsafe { cvt_r(ffi::DjtgSetAuxReset(self.0.raw(), reset, enable_output)) }
    }

    pub fn enable_trans_buffering(&self, enable: bool) -> Result<()> {
        let enable = enable as ffi::BOOL;
        unsafe { cvt_r(ffi::DjtgEnableTransBuffering(self.0.raw(), enable)) }
    }

    pub fn sync_buffer(&self) -> Result<()> {
        unsafe { cvt_r(ffi::DjtgSyncBuffer(self.0.raw())) }
    }

    // misc. functions
    pub fn wait(&self, wait: &Duration) -> Result<Duration> {
        let wait = as_micros(wait) as u32;
        let mut waited: u32 = 0;
        unsafe {
            cvt_r(ffi::DjtgWait(self.0.raw(), wait, &mut waited))?;
        }
        Ok(Duration::new(0, waited))
    }

    // gpio functions
    pub fn gpio_mask(&self) -> Result<(u32, u32)> {
        let mut mask_out: u32 = 0;
        let mut mask_in: u32 = 0;
        unsafe {
            cvt_r(ffi::DjtgGetGpioMask(
                self.0.raw(),
                &mut mask_out,
                &mut mask_in,
            ))?;
        }
        Ok((mask_out, mask_in))
    }

    pub fn gpio_dir(&self) -> Result<u32> {
        let mut dir: u32 = 0;
        unsafe { cvt_r(ffi::DjtgGetGpioDir(self.0.raw(), &mut dir))? }
        Ok(dir)
    }

    pub fn gpio_state(&self) -> Result<u32> {
        let mut state: u32 = 0;
        unsafe {
            cvt_r(ffi::DjtgGetGpioState(self.0.raw(), &mut state))?;
        }
        Ok(state)
    }

    pub fn set_gpio_dir(&self, dir: u32) -> Result<u32> {
        let mut actual: u32 = 0;
        unsafe {
            cvt_r(ffi::DjtgSetGpioDir(self.0.raw(), dir, &mut actual))?;
        }
        Ok(actual)
    }

    pub fn set_gpio_state(&self, state: u32) -> Result<()> {
        unsafe { cvt_r(ffi::DjtgSetGpioState(self.0.raw(), state)) }
    }

    // overlapped functions
    pub fn put_tdi_bits(
        &self,
        tms: bool,
        send: &[u8],
        bit_count: usize,
        overlap: bool,
    ) -> Result<()> {
        if bit_count > u32::max_value() as usize && bit_count > send.len() * 8 {
            return Err(INVALID_PARAMETER)?;
        }
        unsafe {
            cvt_r(ffi::DjtgPutTdiBits(
                self.0.raw(),
                tms as ffi::BOOL,
                send.as_ptr() as *mut u8,
                ptr::null_mut(),
                bit_count as u32,
                overlap as ffi::BOOL,
            ))
        }
    }

    pub fn put_tms_bits(
        &self,
        tdi: bool,
        send: &[u8],
        bit_count: usize,
        overlap: bool,
    ) -> Result<()> {
        if bit_count > u32::max_value as usize && bit_count > send.len() * 8 {
            return Err(INVALID_PARAMETER)?;
        }
        unsafe {
            cvt_r(ffi::DjtgPutTmsBits(
                self.0.raw(),
                tdi as ffi::BOOL,
                send.as_ptr() as *mut u8,
                ptr::null_mut(),
                bit_count as u32,
                overlap as ffi::BOOL,
            ))
        }
    }

    pub fn put_tms_tdi_bits(&self, send: &[u8], pair_count: usize, overlap: bool) -> Result<()> {
        if pair_count > u32::max_value as usize && pair_count > send.len() * 4 {
            return Err(INVALID_PARAMETER)?;
        }
        unsafe {
            cvt_r(ffi::DjtgPutTmsTdiBits(
                self.0.raw(),
                send.as_ptr() as *mut u8,
                ptr::null_mut(),
                pair_count as u32,
                overlap as ffi::BOOL,
            ))
        }
    }

    pub fn tdo_bits(
        &self,
        tdi: bool,
        tms: bool,
        recv: &mut [u8],
        bit_count: usize,
        overlap: bool,
    ) -> Result<()> {
        if bit_count > u32::max_value as usize && bit_count > recv.len() * 8 {
            return Err(INVALID_PARAMETER)?;
        }
        unsafe {
            cvt_r(ffi::DjtgGetTdoBits(
                self.0.raw(),
                tdi as ffi::BOOL,
                tms as ffi::BOOL,
                recv.as_mut_ptr(),
                bit_count as u32,
                overlap as ffi::BOOL,
            ))
        }
    }

    pub fn clock_tick(&self, tms: bool, tdi: bool, cycles: usize, overlap: bool) -> Result<()> {
        if cycles > u32::max_value as usize {
            return Err(INVALID_PARAMETER)?;
        }
        unsafe {
            cvt_r(ffi::DjtgClockTck(
                self.0.raw(),
                tms as ffi::BOOL,
                tdi as ffi::BOOL,
                cycles as u32,
                overlap as ffi::BOOL,
            ))
        }
    }

    pub fn batch(&self, send: &[u8], recv: &mut [u8], overlap: bool) -> Result<()> {
        if send.len() > u32::max_value as usize || recv.len() > u32::max_value as usize {
            return Err(INVALID_PARAMETER)?;
        }
        unsafe {
            cvt_r(ffi::DjtgBatch(
                self.0.raw(),
                send.len() as u32,
                send.as_ptr() as *mut u8,
                recv.len() as u32,
                recv.as_mut_ptr(),
                overlap as ffi::BOOL,
            ))
        }
    }

    // 1149.7-2009 configuration functions
    pub fn scan_format(&self) -> Result<(ScanFormat, bool)> {
        let mut fmt: u8 = 0;
        let mut shift_xr: ffi::BOOL = 0;
        unsafe {
            cvt_r(ffi::DjtgGetScanFormat(
                self.0.raw(),
                &mut fmt,
                &mut shift_xr,
            ))?;
        }
        let fmt = match fmt {
            ffi::jtgsfJScan0 => ScanFormat::JScan0,
            ffi::jtgsfJScan1 => ScanFormat::JScan1,
            ffi::jtgsfJScan2 => ScanFormat::JScan2,
            ffi::jtgsfJScan3 => ScanFormat::JScan3,
            ffi::jtgsfMScan => ScanFormat::MScan,
            ffi::jtgsfOScan0 => ScanFormat::OScan0,
            ffi::jtgsfOScan1 => ScanFormat::OScan1,
            ffi::jtgsfOScan2 => ScanFormat::OScan2,
            ffi::jtgsfOScan3 => ScanFormat::OScan3,
            ffi::jtgsfOScan4 => ScanFormat::OScan4,
            ffi::jtgsfOScan5 => ScanFormat::OScan5,
            ffi::jtgsfOScan6 => ScanFormat::OScan6,
            ffi::jtgsfOScan7 => ScanFormat::OScan7,
            _ => ScanFormat::None,
        };
        Ok((fmt, shift_xr > 0))
    }

    pub fn ready_count(&self) -> Result<(usize, usize)> {
        let mut ready_count: u8 = 0;
        let mut reattempt_count: u32 = 0;
        unsafe {
            cvt_r(ffi::DjtgGetReadyCnt(
                self.0.raw(),
                &mut ready_count,
                &mut reattempt_count,
            ))?;
        }
        Ok((ready_count as usize, reattempt_count as usize))
    }

    pub fn delay_count(&self) -> Result<(usize, bool)> {
        let mut bit_delay: u32 = 0;
        let mut reset: ffi::BOOL = 0;
        unsafe {
            cvt_r(ffi::DjtgGetDelayCnt(
                self.0.raw(),
                &mut bit_delay,
                &mut reset,
            ))?;
        }
        Ok((bit_delay as usize, reset > 0))
    }

    pub fn set_scan_format(&self, format: ScanFormat, shift_xr: bool) -> Result<()> {
        unsafe {
            cvt_r(ffi::DjtgSetScanFormat(
                self.0.raw(),
                format as u8,
                shift_xr as ffi::BOOL,
            ))
        }
    }

    pub fn set_ready_count(&self, ready_bits: u8) -> Result<(usize, usize)> {
        let mut req: u32 = 0;
        let mut set: u32 = 0;
        unsafe {
            cvt_r(ffi::DjtgSetReadyCnt(
                self.0.raw(),
                ready_bits,
                &mut req,
                &mut set,
            ))?;
        }
        Ok((req as usize, set as usize))
    }

    pub fn set_delay_count(&self, delay: usize, reset: bool) -> Result<usize> {
        if delay > u32::max_value as usize {
            return Err(INVALID_PARAMETER)?;
        }
        let mut actual: u32 = 0;
        unsafe {
            cvt_r(ffi::DjtgSetDelayCnt(
                self.0.raw(),
                delay as u32,
                &mut actual,
                reset as ffi::BOOL,
            ))?;
        }
        Ok(actual as usize)
    }

    // 1149.7-2009 misc. functions
    pub fn check_packet(&self, nop_extra: u8, reset: bool, overlap: bool) -> Result<()> {
        unsafe {
            cvt_r(ffi::DjtgCheckPacket(
                self.0.raw(),
                nop_extra,
                reset as ffi::BOOL,
                overlap as ffi::BOOL,
            ))
        }
    }

    pub fn escape(&self, escape_count: u8, overlap: bool) -> Result<()> {
        unsafe {
            cvt_r(ffi::DjtgEscape(
                self.0.raw(),
                escape_count,
                overlap as ffi::BOOL,
            ))
        }
    }
}

impl Drop for Djtag {
    fn drop(&mut self) {
        let _ = disable(&self.0);
    }
}

fn enable(hif: &DeviceHandle) -> Result<()> {
    unsafe { cvt_r(ffi::DjtgEnable(hif.raw())) }
}

fn disable(hif: &DeviceHandle) -> Result<()> {
    unsafe { cvt_r(ffi::DjtgDisable(hif.raw())) }
}

fn as_nanos(dur: &Duration) -> u64 {
    dur.as_secs() * 1_000_000_000 + dur.subsec_nanos() as u64
}

fn as_micros(dur: &Duration) -> u64 {
    as_nanos(dur) / 1_000
}
