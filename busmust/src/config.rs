use crate::{Error, ReceiveFilters, Result};

/// Controller operating mode.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Participate normally, including CAN acknowledgements.
    #[default]
    Normal,
    /// Receive without transmitting frames or acknowledgements.
    ListenOnly,
    /// Loop transmitted frames back internally.
    InternalLoopback,
}

/// CAN FD wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdMode {
    /// Standard ISO CAN FD.
    Iso,
    /// Legacy non-ISO CAN FD.
    NonIso,
}

/// Built-in bus termination.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Termination {
    /// Keep the adapter's current setting.
    #[default]
    Unchanged,
    /// Disconnect the termination resistor.
    Disabled,
    /// Connect the 120-ohm resistor.
    Ohms120,
}

/// CAN channel settings. Validated before any USB operation.
///
/// Bitrates use **bits per second**, not kbit/s. Sample points use whole
/// percentages because the adapter protocol cannot represent fractions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelConfig {
    /// Nominal bitrate; a positive multiple of 1,000, up to 65,535,000.
    pub bitrate: u32,
    /// CAN FD data bitrate, with the same range as `bitrate`.
    pub data_bitrate: u32,
    /// Nominal sample point, 1 through 99 percent.
    pub sample_point: u8,
    /// Data sample point, 1 through 99 percent.
    pub data_sample_point: u8,
    /// `None` selects classic CAN; `Some` enables CAN FD.
    pub fd: Option<FdMode>,
    /// Normal, listen-only, or internal loopback operation.
    pub mode: Mode,
    /// Built-in termination setting.
    pub termination: Termination,
    /// Disable automatic retransmission of unacknowledged frames.
    pub one_shot: bool,
    /// Request and return hardware echoes of transmitted frames.
    pub receive_own_messages: bool,
    /// Acceptance filters, enforced in hardware and on receipt; accepts all by default.
    pub receive_filters: ReceiveFilters,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            bitrate: 500_000,
            data_bitrate: 2_000_000,
            sample_point: 87,
            data_sample_point: 80,
            fd: None,
            mode: Mode::Normal,
            termination: Termination::Unchanged,
            one_shot: false,
            receive_own_messages: false,
            receive_filters: ReceiveFilters::AcceptAll,
        }
    }
}

impl ChannelConfig {
    /// Check whether settings fit the adapter protocol.
    /// Hardware may impose additional limits on supported bit timings.
    pub fn validate(&self) -> Result<()> {
        for rate in [self.bitrate, self.data_bitrate] {
            if rate == 0 || rate % 1000 != 0 || rate > 65_535_000 {
                return Err(Error::InvalidArgument(
                    "bitrates must be positive whole kbit/s fitting u16",
                ));
            }
        }
        if !(1..100).contains(&self.sample_point) || !(1..100).contains(&self.data_sample_point) {
            return Err(Error::InvalidArgument(
                "sample points must be 1 through 99 percent",
            ));
        }
        Ok(())
    }
    pub(crate) fn bitrate_payload(&self) -> [u8; 12] {
        let mut data = [0; 12];
        data[..2].copy_from_slice(&((self.bitrate / 1000) as u16).to_le_bytes());
        let rate = if self.fd.is_some() {
            self.data_bitrate
        } else {
            self.bitrate
        };
        data[2..4].copy_from_slice(&((rate / 1000) as u16).to_le_bytes());
        data[4] = self.sample_point;
        data[5] = self.data_sample_point;
        data
    }
    pub(crate) fn wire_mode(&self) -> u16 {
        let base = match self.mode {
            Mode::InternalLoopback => 2,
            Mode::ListenOnly => 3,
            Mode::Normal if self.fd.is_some() => 0,
            Mode::Normal => 6,
        };
        base | if self.fd == Some(FdMode::NonIso) {
            8
        } else {
            0
        } | if self.one_shot { 16 } else { 0 }
    }
}
