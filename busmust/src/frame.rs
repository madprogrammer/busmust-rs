// SPDX-License-Identifier: GPL-2.0-or-later
// Rust adaptation and changes: 2026-10-07. See NOTICE.md for provenance.

use crate::{Error, Result};

/// A validated 11-bit standard or 29-bit extended CAN identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id {
    raw: u32,
    extended: bool,
}

impl Id {
    /// Construct an 11-bit identifier, rejecting values above `0x7ff`.
    pub fn standard(raw: u16) -> Result<Self> {
        if raw > 0x7ff {
            return Err(Error::InvalidArgument("standard ID exceeds 11 bits"));
        }
        Ok(Self {
            raw: u32::from(raw),
            extended: false,
        })
    }
    /// Construct a 29-bit identifier, rejecting values above `0x1fff_ffff`.
    pub fn extended(raw: u32) -> Result<Self> {
        if raw > 0x1fff_ffff {
            return Err(Error::InvalidArgument("extended ID exceeds 29 bits"));
        }
        Ok(Self {
            raw,
            extended: true,
        })
    }
    /// The numeric arbitration identifier, without flags.
    pub const fn as_raw(self) -> u32 {
        self.raw
    }
    /// Whether this identifier uses the extended format.
    pub const fn is_extended(self) -> bool {
        self.extended
    }
}

/// CAN FD transmit options. These cannot be applied to classic or remote frames.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FdOptions {
    /// Switch to the configured data bitrate for the data phase.
    pub bitrate_switch: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Classic,
    Remote,
    Fd(FdOptions),
}

/// An immutable, validated CAN frame with inline payload storage.
///
/// CAN FD payloads are zero-padded at construction to a legal wire length:
/// 0–8, 12, 16, 20, 24, 32, 48, or 64 bytes. [`Self::data`] includes padding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    id: Id,
    format: Format,
    len: u8,
    data: [u8; 64],
    pub(crate) error_state_indicator: bool,
}

pub(crate) const LENGTHS: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 12, 16, 20, 24, 32, 48, 64];

impl Frame {
    /// Construct a classic CAN data frame containing at most eight bytes.
    pub fn classic(id: Id, data: &[u8]) -> Result<Self> {
        Self::new(id, Format::Classic, data, 8)
    }
    /// Construct a CAN FD data frame containing at most 64 bytes.
    pub fn fd(id: Id, data: &[u8], options: FdOptions) -> Result<Self> {
        Self::new(id, Format::Fd(options), data, 64)
    }
    /// Construct a classic remote request for zero to eight bytes, with no data.
    pub fn remote(id: Id, requested_len: u8) -> Result<Self> {
        if requested_len > 8 {
            return Err(Error::InvalidArgument("remote length exceeds eight bytes"));
        }
        Ok(Self {
            id,
            format: Format::Remote,
            len: requested_len,
            data: [0; 64],
            error_state_indicator: false,
        })
    }
    fn new(id: Id, format: Format, data: &[u8], max: usize) -> Result<Self> {
        if data.len() > max {
            return Err(Error::InvalidArgument("payload exceeds frame capacity"));
        }
        let len = LENGTHS
            .iter()
            .copied()
            .find(|&n| usize::from(n) >= data.len())
            .unwrap();
        let mut frame = Self {
            id,
            format,
            len,
            data: [0; 64],
            error_state_indicator: false,
        };
        frame.data[..data.len()].copy_from_slice(data);
        Ok(frame)
    }
    /// Arbitration identifier and identifier format.
    pub const fn id(&self) -> Id {
        self.id
    }
    /// Payload bytes; empty for remote requests.
    pub fn data(&self) -> &[u8] {
        &self.data[..if self.is_remote() {
            0
        } else {
            usize::from(self.len)
        }]
    }
    /// Payload length in bytes, or requested length for a remote frame.
    pub const fn len(&self) -> usize {
        self.len as usize
    }
    /// Whether the payload or requested length is zero.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Whether this is a classic remote request.
    pub const fn is_remote(&self) -> bool {
        matches!(self.format, Format::Remote)
    }
    /// Whether this uses CAN FD.
    pub const fn is_fd(&self) -> bool {
        matches!(self.format, Format::Fd(_))
    }
    /// Whether a received CAN FD frame reports an error-passive transmitter (ESI).
    /// Always false for locally constructed frames and classic CAN. The physical
    /// controller supplies ESI during transmission; applications cannot force it.
    pub const fn is_error_passive(&self) -> bool {
        self.error_state_indicator
    }
    /// CAN FD transmit options, or `None` for classic CAN.
    pub const fn fd_options(&self) -> Option<FdOptions> {
        match self.format {
            Format::Fd(flags) => Some(flags),
            _ => None,
        }
    }
    pub(crate) fn dlc(&self) -> u8 {
        LENGTHS.iter().position(|&n| n == self.len).unwrap() as u8
    }
}

/// Original device timestamps, without a host-clock estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp {
    /// Microsecond device counter. Wraps every 2^32 microseconds (~71.6 minutes).
    pub device_micros: u32,
    /// Gen3 UTC microseconds since Unix epoch, when a nonzero timestamp tail exists.
    pub unix_micros: Option<u64>,
}

/// A received CAN frame or a hardware transmit echo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceivedFrame {
    /// Zero-based physical CAN channel.
    pub channel: u8,
    /// CAN frame, including its flags and payload.
    pub frame: Frame,
    /// Device-provided timestamps.
    pub timestamp: Timestamp,
    /// True for transmit echoes, false for bus reception (including loopback).
    pub is_echo: bool,
}
