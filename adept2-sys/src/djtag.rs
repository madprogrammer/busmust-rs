/*    This header file contains the interface declarations for the      */
/*    applications programming interface to the Digilent djtg.DLL       */
/*                                                                      */
/*    This DLL provides API services to provide the JTAG                */
/*    application protocol for Adept2.                                  */

// Define port properties bits for JTAG ports.
use dpcdecl::*;
use std::os::raw::c_char;

/// port supports set speed call
pub const dprpJtgSetSpeed: DPRP = 0x00000001;
/// device fully implements DjtgSetTmsTdiTck
pub const dprpJtgSetPinState: DPRP = 0x00000002;
/// port supports transaction buffering
pub const dprpJtgTransBuffering: DPRP = 0x00000004;
/// port supports DjtgWait
pub const dprpJtgWait: DPRP = 0x00000008;
/// port supports DjtgSetDelayCnt and DjtgGetDelayCnt
pub const dprpJtgDelayCnt: DPRP = 0x00000010;
/// port supports DjtgSetReadyCnt and DjtgGetReadyCnt
pub const dprpJtgReadyCnt: DPRP = 0x00000020;
/// port supports DjtgEscape
pub const dprpJtgEscape: DPRP = 0x00000040;
/// port supports the MScan format
pub const dprpJtgMScan: DPRP = 0x00000080;
/// port supports the OScan0 format
pub const dprpJtgOScan0: DPRP = 0x00000100;
/// port supports the OScan1 format
pub const dprpJtgOScan1: DPRP = 0x00000200;
/// port supports the OScan2 format
pub const dprpJtgOScan2: DPRP = 0x00000400;
/// port supports the OScan3 format
pub const dprpJtgOScan3: DPRP = 0x00000800;
/// port supports the OScan4 format
pub const dprpJtgOScan4: DPRP = 0x00001000;
/// port supports the OScan5 format
pub const dprpJtgOScan5: DPRP = 0x00002000;
/// port supports the OScan6 format
pub const dprpJtgOScan6: DPRP = 0x00004000;
/// port supports the OScan7 format
pub const dprpJtgOScan7: DPRP = 0x00008000;
/// port supports DjtgCheckPacket
pub const dprpJtgCheckPacket: DPRP = 0x00010000;
/// port supports DjtgBatch
pub const dprpJtgBatch: DPRP = 0x00020000;
/// port supports DjtgSetAuxReset
pub const dprpJtgSetAuxReset: DPRP = 0x00040000;
/// port supports DjtgSetGpioState and related functions
pub const dprpJtgSetGetGpio: DPRP = 0x00080000;

// Define the symbols used to specify the scan format when calling
// DjtgSetScanFormat.

pub const jtgsfNone: u8 = 0;
pub const jtgsfJScan0: u8 = 1;
pub const jtgsfJScan1: u8 = 2;
pub const jtgsfJScan2: u8 = 3;
pub const jtgsfJScan3: u8 = 4;
pub const jtgsfMScan: u8 = 5;
pub const jtgsfOScan0: u8 = 6;
pub const jtgsfOScan1: u8 = 7;
pub const jtgsfOScan2: u8 = 8;
pub const jtgsfOScan3: u8 = 9;
pub const jtgsfOScan4: u8 = 10;
pub const jtgsfOScan5: u8 = 11;
pub const jtgsfOScan6: u8 = 12;
pub const jtgsfOScan7: u8 = 13;

// Define the symbols used to specify the edge count when calling
// DjtgEscape.

/// Number of edges for a Custom Escape
pub const cedgeJtgCustom: u8 = 2;
/// Number of edges for a Deselection Escape
pub const cedgeJtgDeselect: u8 = 4;
/// Number of edges for a Selection Escape
pub const cedgeJtgSelect: u8 = 6;
/// Number of edges for a Reset Escape
pub const cedgeJtgReset: u8 = 8;

/* ------------------------------------------------------------ */
/*              JTG Batch Command Declarations                  */
/* ------------------------------------------------------------ */

pub const jcbSetTmsTdiTck: u32 = 1;
pub const jcbGetTmsTdiTdoTck: u32 = 2;
pub const jcbPutTms: u32 = 3;
pub const jcbPutTmsGetTdo: u32 = 4;
pub const jcbPutTdi: u32 = 5;
pub const jcbPutTdiGetTdo: u32 = 6;
pub const jcbGetTdo: u32 = 7;
pub const jcbClockTck: u32 = 8;
pub const jcbWaitUs: u32 = 9;
pub const jcbSetAuxReset: u32 = 10;
pub const jcbGetGpioMask: u32 = 11;
pub const jcbSetGpioDir: u32 = 12;
pub const jcbGetGpioDir: u32 = 13;
pub const jcbSetGpioState: u32 = 14;
pub const jcbGetGpioState: u32 = 15;

pub const djbpWaitUs: u32 = 0x00000001;
pub const djbpSetAuxReset: u32 = 0x00000002;
pub const djbpSetGetGpio: u32 = 0x00000004;

extern "C" {
    pub fn DjtgGetVersion(szVersion: *mut c_char) -> BOOL;
    pub fn DjtgGetPortCount(hif: HIF, pcprt: *mut i32) -> BOOL;
    pub fn DjtgGetPortProperties(hif: HIF, prtReq: i32, pdprp: *mut u32) -> BOOL;
    pub fn DjtgGetBatchProperties(hif: HIF, prtReq: i32, pdjbp: *mut u32) -> BOOL;
    pub fn DjtgEnable(hif: HIF) -> BOOL;
    pub fn DjtgEnableEx(hif: HIF, prtReq: i32) -> BOOL;
    pub fn DjtgDisable(hif: HIF) -> BOOL;

    // configuration functions
    pub fn DjtgGetSpeed(hif: HIF, pfrqCur: *mut u32) -> BOOL;
    pub fn DjtgSetSpeed(hif: HIF, frqReq: u32, pfrqSet: *mut u32) -> BOOL;
    pub fn DjtgSetTmsTdiTck(hif: HIF, fTms: BOOL, fTdi: BOOL, fTck: BOOL) -> BOOL;
    pub fn DjtgGetTmsTdiTdoTck(
        hif: HIF,
        pfTms: *mut BOOL,
        pfTdi: *mut BOOL,
        pfTdo: *mut BOOL,
        pfTck: *mut BOOL,
    ) -> BOOL;
    pub fn DjtgSetAuxReset(hif: HIF, fReset: BOOL, fEnOutput: BOOL) -> BOOL;
    pub fn DjtgEnableTransBuffering(hif: HIF, fEnable: BOOL) -> BOOL;
    pub fn DjtgSyncBuffer(hif: HIF) -> BOOL;

    // misc. functions
    pub fn DjtgWait(hif: HIF, tusWait: u32, ptusWaited: *mut u32) -> BOOL;

    // gpio functions
    pub fn DjtgGetGpioMask(hif: HIF, pfsMaskOut: *mut u32, pfsMaskIn: *mut u32) -> BOOL;
    pub fn DjtgSetGpioDir(hif: HIF, fsDirReq: u32, pfsDirSet: *mut u32) -> BOOL;
    pub fn DjtgGetGpioDir(hif: HIF, pfsDir: *mut u32) -> BOOL;
    pub fn DjtgSetGpioState(hif: HIF, fsState: u32) -> BOOL;
    pub fn DjtgGetGpioState(hif: HIF, pfsState: *mut u32) -> BOOL;

    // overlapped functions
    pub fn DjtgPutTdiBits(
        hif: HIF,
        fTms: BOOL,
        rgbSnd: *mut u8,
        rgbRcv: *mut u8,
        cbits: u32,
        fOverlap: BOOL,
    ) -> BOOL;
    pub fn DjtgPutTmsBits(
        hif: HIF,
        fTdi: BOOL,
        rgbSnd: *mut u8,
        rgbRcv: *mut u8,
        cbits: u32,
        fOverlap: BOOL,
    ) -> BOOL;
    pub fn DjtgPutTmsTdiBits(
        hif: HIF,
        rgbSnd: *mut u8,
        rgbRcv: *mut u8,
        cbitpairs: u32,
        fOverlap: BOOL,
    ) -> BOOL;
    pub fn DjtgGetTdoBits(
        hif: HIF,
        fTdi: BOOL,
        fTms: BOOL,
        rgbRcv: *mut u8,
        cbits: u32,
        fOverlap: BOOL,
    ) -> BOOL;
    pub fn DjtgClockTck(hif: HIF, fTms: BOOL, fTdi: BOOL, cclk: u32, fOverlap: BOOL) -> BOOL;
    pub fn DjtgBatch(
        hif: HIF,
        cbSnd: u32,
        rgbSnd: *mut u8,
        cbRcv: u32,
        rgbRcv: *mut u8,
        fOverlap: BOOL,
    ) -> BOOL;

    // 1149.7-2009 configuration functions
    pub fn DjtgSetScanFormat(hif: HIF, jtgsfFmt: u8, fShiftXR: BOOL) -> BOOL;
    pub fn DjtgGetScanFormat(hif: HIF, pjtgsfFmt: *mut u8, pfShiftXR: *mut BOOL) -> BOOL;
    pub fn DjtgSetReadyCnt(
        hif: HIF,
        cbitRdy: u8,
        pcretOutReq: *mut u32,
        pcretOutSet: *mut u32,
    ) -> BOOL;
    pub fn DjtgGetReadyCnt(hif: HIF, pcbitRdy: *mut u8, pcretOut: *mut u32) -> BOOL;
    pub fn DjtgSetDelayCnt(hif: HIF, cbitDlyReq: u32, pcbitDlySet: *mut u32, fReset: BOOL) -> BOOL;
    pub fn DjtgGetDelayCnt(hif: HIF, pcbitDly: *mut u32, pfReset: *mut BOOL) -> BOOL;

    // 1149.7-2009 misc. functions
    pub fn DjtgCheckPacket(hif: HIF, cedgeNop: u8, fReset: BOOL, fOverlap: BOOL) -> BOOL;
    pub fn DjtgEscape(hif: HIF, cedgeEsc: u8, fOverlap: BOOL) -> BOOL;
}
