// SPDX-License-Identifier: GPL-2.0-or-later
// Rust adaptation and changes: 2026-10-07. See NOTICE.md for provenance.

use crate::{Error, Frame, Id, Result};

/// An identifier/mask acceptance filter, optionally constrained by frame flags.
///
/// A frame matches when its identifier format matches and
/// `(frame.id & mask) == (filter.id & mask)`. Multiple filters are ORed.
/// Filters are sent to the hardware and also checked on receipt, because older
/// Gen2 firmware ignores some flag constraints in its basic hardware filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Filter {
    id: Id,
    mask: u32,
    flags_mask: u8,
    flags_value: u8,
}

impl Filter {
    /// Match a standard or extended identifier with the given mask.
    /// Mask bits outside the selected identifier format are rejected.
    pub fn new(id: Id, mask: u32) -> Result<Self> {
        let maximum = if id.is_extended() { 0x1fff_ffff } else { 0x7ff };
        if mask > maximum {
            return Err(Error::InvalidArgument(
                "filter mask exceeds identifier width",
            ));
        }
        Ok(Self {
            id,
            mask,
            flags_mask: 1,
            flags_value: u8::from(id.is_extended()),
        })
    }
    /// Match exactly this identifier and identifier format.
    pub fn exact(id: Id) -> Self {
        Self {
            id,
            mask: if id.is_extended() { 0x1fff_ffff } else { 0x7ff },
            flags_mask: 1,
            flags_value: u8::from(id.is_extended()),
        }
    }
    /// Require remote (`true`) or data (`false`) frames.
    pub fn with_remote(mut self, remote: bool) -> Self {
        self.set_flag(2, remote);
        self
    }
    /// Require CAN FD (`true`) or classic CAN (`false`) frames.
    pub fn with_fd(mut self, fd: bool) -> Self {
        self.set_flag(8, fd);
        self
    }
    /// Require (`true`) or exclude (`false`) bitrate switching.
    pub fn with_bitrate_switch(mut self, enabled: bool) -> Self {
        self.set_flag(4, enabled);
        self
    }
    fn set_flag(&mut self, flag: u8, enabled: bool) {
        self.flags_mask |= flag;
        if enabled {
            self.flags_value |= flag;
        } else {
            self.flags_value &= !flag;
        }
    }
    /// Identifier being matched. Bits outside the mask are ignored.
    pub fn id(&self) -> Id {
        self.id
    }
    /// Identifier bits participating in the comparison.
    pub fn mask(&self) -> u32 {
        self.mask
    }
    /// Whether a frame satisfies this filter's identifier and flag constraints.
    pub fn matches(&self, frame: &Frame) -> bool {
        let mut flags = u8::from(frame.id().is_extended());
        if frame.is_remote() {
            flags |= 2;
        }
        if let Some(options) = frame.fd_options() {
            flags |= 8;
            if options.bitrate_switch {
                flags |= 4;
            }
        }
        flags & self.flags_mask == self.flags_value
            && frame.id().as_raw() & self.mask == self.id.as_raw() & self.mask
    }
    pub(crate) fn encode(self) -> [u8; 32] {
        let mut bytes = [0; 32];
        bytes[0] = 1; // Basic filter.
        bytes[2] = self.flags_mask;
        bytes[3] = self.flags_value;
        let pack = |raw: u32| {
            if self.id.is_extended() {
                ((raw & 0x3ffff) << 11) | (raw >> 18)
            } else {
                raw
            }
        };
        bytes[8..12].copy_from_slice(&pack(self.mask).to_le_bytes());
        bytes[12..16].copy_from_slice(&pack(self.id.as_raw() & self.mask).to_le_bytes());
        bytes
    }
}

/// Acceptance policy backed by the adapter's two hardware filter slots.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ReceiveFilters {
    /// Receive every frame, including both identifier formats.
    #[default]
    AcceptAll,
    /// Receive frames matching one filter.
    Match(Filter),
    /// Receive frames matching either filter.
    Either(Filter, Filter),
}

impl ReceiveFilters {
    /// Whether a frame satisfies this acceptance policy.
    pub fn matches(&self, frame: &Frame) -> bool {
        match self {
            Self::AcceptAll => true,
            Self::Match(filter) => filter.matches(frame),
            Self::Either(first, second) => first.matches(frame) || second.matches(frame),
        }
    }
    pub(crate) fn encode(self) -> [[u8; 32]; 2] {
        let mut slots = [[0; 32]; 2];
        match self {
            Self::AcceptAll => slots[0][0] = 1,
            Self::Match(filter) => slots[0] = filter.encode(),
            Self::Either(first, second) => {
                slots[0] = first.encode();
                slots[1] = second.encode();
            }
        }
        slots
    }
}
