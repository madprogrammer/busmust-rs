// SPDX-License-Identifier: GPL-2.0-or-later
// Rust adaptation and changes: 2026-10-07. See NOTICE.md for provenance.

use std::fmt;

/// A driver operation result.
pub type Result<T> = std::result::Result<T, Error>;

/// Configuration, protocol, or USB failure.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The operating system could not enumerate, open, or claim the adapter.
    Usb(nusb::Error),
    /// A USB transfer failed. A failed send may have partially completed.
    Transfer(nusb::transfer::TransferError),
    /// A bulk send exceeded its timeout; the frame may have partially transferred.
    Timeout,
    /// An argument cannot be represented by CAN or the adapter protocol.
    InvalidArgument(&'static str),
    /// The adapter returned malformed data; reopen the connection.
    Protocol(&'static str),
    /// A transfer completed with fewer bytes than required.
    ShortTransfer {
        /// Required number of bytes.
        expected: usize,
        /// Transferred number of bytes.
        actual: usize,
    },
    /// The selected channel is outside this adapter's range.
    InvalidChannel(u8),
    /// Configure the channel before using it.
    ChannelInactive(u8),
    /// Transmitting is forbidden in listen-only mode.
    ListenOnly,
    /// The controller is bus-off; recover it before sending again.
    BusOff(u8),
    /// Recovery could not safely drain the receive stream; reopen the adapter.
    RecoveryFailed(&'static str),
    /// A prior receive error made the USB stream unusable; reopen the device.
    ReceiveFailed,
    /// A prior write may have left an incomplete envelope; reopen the device.
    TransmitFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usb(e) => write!(f, "USB device error: {e}"),
            Self::Timeout => {
                f.write_str("USB send timed out; the frame may have partially transferred")
            }
            Self::Transfer(e) => write!(f, "USB transfer error: {e}"),
            Self::InvalidArgument(s) => write!(f, "invalid argument: {s}"),
            Self::Protocol(s) => write!(f, "invalid USB data: {s}"),
            Self::ShortTransfer { expected, actual } => {
                write!(f, "short USB transfer: {actual} of {expected} bytes")
            }
            Self::InvalidChannel(c) => write!(f, "channel {c} is outside the adapter's range"),
            Self::ChannelInactive(c) => write!(f, "channel {c} is not configured"),
            Self::ListenOnly => f.write_str("cannot transmit in listen-only mode"),
            Self::BusOff(c) => write!(f, "channel {c} is bus-off"),
            Self::RecoveryFailed(s) => write!(f, "bus-off recovery failed: {s}"),
            Self::TransmitFailed => f.write_str("transmit stream failed; reopen the device"),
            Self::ReceiveFailed => f.write_str("receive stream failed; reopen the device"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Usb(e) => Some(e),
            Self::Transfer(e) => Some(e),
            _ => None,
        }
    }
}

impl From<nusb::Error> for Error {
    fn from(error: nusb::Error) -> Self {
        Self::Usb(error)
    }
}
impl From<nusb::transfer::TransferError> for Error {
    fn from(error: nusb::transfer::TransferError) -> Self {
        Self::Transfer(error)
    }
}

pub(crate) fn check_length(actual: usize, expected: usize) -> Result<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(Error::ShortTransfer { expected, actual })
    }
}
