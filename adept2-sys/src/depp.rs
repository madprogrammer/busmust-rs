use dpcdecl::*;
use std::os::raw::c_char;

extern "C" {
    /* Basic interface functions. */
    pub fn DeppGetVersion(szVersion: *mut c_char) -> BOOL;
    pub fn DeppGetPortCount(hif: HIF, pcprt: *mut i32) -> BOOL;
    pub fn DeppGetPortProperties(hif: HIF, prtReq: i32, pdprp: *mut u32) -> BOOL;
    pub fn DeppEnable(hif: HIF) -> BOOL;
    pub fn DeppEnableEx(hif: HIF, prtReq: i32) -> BOOL;
    pub fn DeppDisable(hif: HIF) -> BOOL;

    /* Data transfer functions */
    pub fn DeppPutReg(hif: HIF, bAddr: u8, bData: u8, fOverlap: BOOL) -> BOOL;
    pub fn DeppGetReg(hif: HIF, bAddr: u8, pbData: *mut u8, fOverlap: BOOL) -> BOOL;
    pub fn DeppPutRegSet(
        hif: HIF,
        pbAddrData: *mut u8,
        nAddrDataPairs: u32,
        fOverlap: BOOL,
    ) -> BOOL;
    pub fn DeppGetRegSet(
        hif: HIF,
        pbAddr: *mut u8,
        pbData: *mut u8,
        cbData: u32,
        fOverlap: BOOL,
    ) -> BOOL;
    pub fn DeppPutRegRepeat(
        hif: HIF,
        bAddr: u8,
        pbData: *mut u8,
        cbData: u32,
        fOverlap: BOOL,
    ) -> BOOL;
    pub fn DeppGetRegRepeat(
        hif: HIF,
        bAddr: u8,
        pbData: *mut u8,
        cbData: u32,
        fOverlap: BOOL,
    ) -> BOOL;

    /* Misc. control functions */
    pub fn DeppSetTimeout(hif: HIF, tnsTimeoutTry: u32, ptnsTimeout: *mut u32) -> BOOL;
}
