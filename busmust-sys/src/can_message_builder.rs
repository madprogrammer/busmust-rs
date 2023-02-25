use std::convert::TryFrom;
use ::{BMCanMessage, BMMessageId};
use ::{BMMessageCtrl, BMTxMessageCtrl};

impl BMCanMessage {
    pub fn builder() -> BMCanMessageBuilder {
        BMCanMessageBuilder::default()
    }
}

/// Builder for [BMCanMessage] structure.
#[derive(Default)]
pub struct BMCanMessageBuilder {
    sid: Option<u16>,
    eid: Option<u32>,
    dlc: Option<u8>,
    ide: Option<bool>,
    rtr: Option<bool>,
    brs: Option<bool>,
    fdf: Option<bool>,
    payload: Vec<u8>
}

impl BMCanMessageBuilder {
    pub fn new(payload: Vec<u8>) -> BMCanMessageBuilder {
        let mut vec = Vec::from(payload);
        vec.resize(64, 0);

        BMCanMessageBuilder {
            sid: None,
            eid: None,
            dlc: None,
            ide: None,
            rtr: None,
            brs: None,
            fdf: None,
            payload: vec
        }
    }

    pub fn sid(mut self, value: u16) -> BMCanMessageBuilder {
        self.sid = Some(value);
        self
    }

    pub fn eid(mut self, value: u32) -> BMCanMessageBuilder {
        self.eid = Some(value);
        self
    }

    pub fn dlc(mut self, value: u8) -> BMCanMessageBuilder {
        self.dlc = Some(value);
        self
    }

    pub fn ide(mut self, value: bool) -> BMCanMessageBuilder {
        self.ide = Some(value);
        self
    }

    pub fn rtr(mut self, value: bool) -> BMCanMessageBuilder {
        self.rtr = Some(value);
        self
    }

    pub fn brs(mut self, value: bool) -> BMCanMessageBuilder {
        self.brs = Some(value);
        self
    }

    pub fn fdf(mut self, value: bool) -> BMCanMessageBuilder {
        self.fdf = Some(value);
        self
    }

    pub fn payload(mut self, value: Vec<u8>) -> BMCanMessageBuilder {
        self.dlc = Some(value.len() as u8);

        let mut vec = Vec::from(value);
        vec.resize(64, 0);

        self.payload = vec;
        self
    }

    pub fn build(self) -> BMCanMessage {
        BMCanMessage {
            mid: BMMessageId::new()
                .with_sid(self.sid.unwrap_or_default())
                .with_eid(self.eid.unwrap_or_default()),
            ctrl: BMMessageCtrl {
                tx: BMTxMessageCtrl::new()
                    .with_dlc(self.dlc.unwrap_or_default())
                    .with_ide(self.ide.unwrap_or_default())
                    .with_rtr(self.rtr.unwrap_or_default())
                    .with_brs(self.brs.unwrap_or_default())
                    .with_fdf(self.fdf.unwrap_or_default()),
            },
            payload: <[u8; 64]>::try_from(self.payload).unwrap()
        }
    }
}
