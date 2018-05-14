use dpcdecl::*;
use std::os::raw::{c_char, c_int, c_void};

pub const tmsWaitInfinite: u32 = 0xFFFFFFFF;

extern "C" {
    pub fn DmgrGetVersion(szVersion: *mut c_char) -> BOOL;
    //DmgrGetLastError returns the last error per process which is updated when a DVC API function fails.
    pub fn DmgrGetLastError() -> ERC;
    pub fn DmgrSzFromErc(erc: c_int, szErc: *mut c_char, szErcMessage: *mut c_char) -> BOOL;

    //OPEN & CLOSE functions
    pub fn DmgrOpen(phif: *mut HIF, szSel: *mut c_char) -> BOOL;
    pub fn DmgrOpenEx(phif: *mut HIF, szSel: *mut c_char, dtpTable: DTP, dtpDisc: DTP) -> BOOL;
    pub fn DmgrClose(hif: HIF) -> BOOL;

    //ENUMERATION functions
    pub fn DmgrEnumDevices(pcdvc: *mut c_int) -> BOOL;
    pub fn DmgrEnumDevicesEx(
        pcdvc: *mut c_int,
        dtpTable: DTP,
        dtpDisc: DTP,
        dinfosel: DINFO,
        pInfoSel: *mut c_void,
    ) -> BOOL;
    pub fn DmgrStartEnum(
        dtpTable: DTP,
        dtpDisc: DTP,
        dinfoSel: DINFO,
        pInfoSel: *mut c_void,
    ) -> BOOL;
    pub fn DmgrIsEnumFinished() -> BOOL;
    pub fn DmgrStopEnum() -> BOOL;
    pub fn DmgrGetEnumCount(pcdvc: *mut c_int) -> BOOL;
    pub fn DmgrGetDvc(idvc: c_int, pdvc: *mut DVC) -> BOOL;
    pub fn DmgrFreeDvcEnum() -> BOOL;

    //TRANSFER status and control functions
    pub fn DmgrGetTransResult(
        hif: HIF,
        pdwDataOut: *mut u32,
        pdwDataIn: *mut u32,
        tmsWait: u32,
    ) -> BOOL;
    pub fn DmgrCancelTrans(hif: HIF) -> BOOL;
    pub fn DmgrSetTransTimeout(hif: HIF, tmsTimeout: u32) -> BOOL;
    pub fn DmgrGetTransTimeout(hif: HIF, ptmsTimeout: *mut u32) -> BOOL;

    pub fn DmgrDvcTblAdd(pdvc: *mut DVC) -> BOOL;
    pub fn DmgrDvcTblRem(szAlias: *mut c_char) -> BOOL;
    pub fn DmgrDvcTblSave() -> BOOL;

    //Device transport type management functions
    pub fn DmgrGetDtpCount() -> c_int;
    pub fn DmgrGetDtpFromIndex(idtp: c_int, pdtp: *mut DTP) -> BOOL;
    pub fn DmgrGetDtpString(dtp: DTP, szDtpString: *mut c_char) -> BOOL;

    //Miscellaneous functions
    pub fn DmgrSetInfo(pdvc: *mut DVC, dinfo: DINFO, pvInfoSet: *mut c_void) -> BOOL;
    pub fn DmgrGetInfo(pdvc: *mut DVC, dinfo: DINFO, pvInfoGet: *mut c_void) -> BOOL;

    pub fn DmgrGetDvcFromHif(hif: HIF, pdvc: *mut DVC) -> BOOL;
}
