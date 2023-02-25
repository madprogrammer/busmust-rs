use BMBitrate;

impl BMBitrate {
    pub fn builder() -> BMBitrateBuilder {
        BMBitrateBuilder::default()
    }
}

/// Builder for [BMBitrate] structure.
#[derive(Default)]
pub struct BMBitrateBuilder {
    /// Nominal bitrate in kbps, default as 500, note this is the only valid bitrate in CAN CLASSIC mode.
    n_bitrate: u16,
    /// Data bitrate in kbps, default as 500, note this is ignored in CAN CLASSIC mode.
    d_bitrate: u16,
    /// Nominal sample position (percentage, 0-100, default as 75
    n_sample_pos: u8,
    /// Data sample position (percentage, 0-100, default as 75
    d_sample_pos: u8
}

impl BMBitrateBuilder {
    pub fn new() -> BMBitrateBuilder {
        BMBitrateBuilder {
            n_bitrate: 500,
            d_bitrate: 2000,
            n_sample_pos: 75,
            d_sample_pos: 75
        }
    }

    pub fn bitrate(mut self, n_bitrate: u16) -> BMBitrateBuilder {
        self.n_bitrate = n_bitrate;
        self
    }

    pub fn data_bitrate(mut self, d_bitrate: u16) -> BMBitrateBuilder {
        self.d_bitrate = d_bitrate;
        self
    }

    pub fn sample_pos(mut self, n_sample_pos: u8) -> BMBitrateBuilder {
        self.n_sample_pos = n_sample_pos;
        self
    }

    pub fn data_sample_pos(mut self, d_sample_pos: u8) -> BMBitrateBuilder {
        self.d_sample_pos = d_sample_pos;
        self
    }

    pub fn build(self) -> BMBitrate {
        BMBitrate {
            n_bitrate: self.n_bitrate,
            d_bitrate: self.d_bitrate,
            n_sample_pos: self.n_sample_pos,
            d_sample_pos: self.d_sample_pos,
            clock_freq: 0,
            reserved: 0,
            n_btr0: 0,
            n_btr1: 0,
            d_btr0: 0,
            d_btr1: 0
        }
    }
}
