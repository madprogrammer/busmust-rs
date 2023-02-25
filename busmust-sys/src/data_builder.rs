use std::convert::TryFrom;
use ::{BMData, BMDataHeader};
use ::{BM_DATA_PAYLOAD_MAX_SIZE, BMDataType};
use BMCanMessage;

impl BMData {
    pub fn builder() -> BMDataBuilder {
        BMDataBuilder::default()
    }
}

/// Builder for [BMData] structure.
#[derive(Default)]
pub struct BMDataBuilder {
    header: BMDataHeader,
    payload: Vec<u8>,
}

impl BMDataBuilder {
    pub fn new(payload: Vec<u8>) -> BMDataBuilder {
        let mut vec = Vec::from(payload);
        vec.resize(BM_DATA_PAYLOAD_MAX_SIZE, 0);

        BMDataBuilder {
            header: BMDataHeader::new().with_kind(BMDataType::Can as u8),
            payload: vec
        }
    }

    pub fn kind(mut self, kind: BMDataType) {
        self.header = BMDataHeader::new()
            .with_kind(kind as u8)
            .with_flags(self.header.flags())
            .with_dchn(self.header.dchn())
            .with_schn(self.header.schn())
    }

    pub fn dst_chn(mut self, dst_chn: u8) {
        self.header = BMDataHeader::new()
            .with_kind(self.header.kind())
            .with_flags(self.header.flags())
            .with_dchn(dst_chn)
            .with_schn(self.header.schn())
    }

    pub fn src_chn(mut self, src_chn: u8) {
        self.header = BMDataHeader::new()
            .with_kind(self.header.kind())
            .with_flags(self.header.flags())
            .with_dchn(self.header.dchn())
            .with_schn(src_chn)
    }

    pub fn payload(mut self, value: Vec<u8>) -> BMDataBuilder {
        let mut vec = Vec::from(value);
        vec.resize(BM_DATA_PAYLOAD_MAX_SIZE, 0);

        self.payload = vec;
        self
    }

    pub fn can_message(mut self, value: BMCanMessage) -> BMDataBuilder {
        self.payload = value.to_bytes();
        self.header = BMDataHeader::new().with_kind(BMDataType::Can as u8);
        self
    }

    pub fn build(self) -> BMData {
        BMData {
            header: self.header,
            length: self.payload.len() as u16,
            timestamp: 0,
            payload: <[u8; BM_DATA_PAYLOAD_MAX_SIZE]>::try_from(self.payload).unwrap(),
        }
    }
}
