// SPDX-License-Identifier: GPL-2.0-or-later
// Rust adaptation and changes: 2026-10-07. See NOTICE.md for provenance.

//! Direct USB access to BUSMUST CAN and CAN FD adapters.
//!
//! A [`Device`] owns one USB interface and all of its CAN channels. Configure
//! channels, send validated [`Frame`]s, and receive frames from all channels in
//! wire order. No vendor SDK, global initialization, or async runtime is needed.
//!
//! ```no_run
//! use busmust::{ChannelConfig, Frame, Id};
//! use std::time::Duration;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let info = busmust::devices()?.next().ok_or("no BUSMUST adapter")?;
//! let mut device = info.open()?;
//! device.configure_channel(0, ChannelConfig::default())?;
//! let frame = Frame::classic(Id::standard(0x123)?, &[1, 2, 3])?;
//! device.send(0, &frame, Duration::from_secs(1))?;
//! if let Some(received) = device.receive(Duration::from_secs(1))? {
//!     println!("channel {}: {:?}", received.channel, received.frame);
//! }
//! device.close()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Platform setup
//!
//! On Linux, grant your user USB access to vendor ID `0810`, for example with a
//! udev rule. If SocketCAN's `bmcan` driver owns the interface, release it or use
//! [`DeviceInfo::open_with_options`] with [`OpenOptions::detach_kernel_driver`].
//! nusb restores the kernel driver when the interface is released on Linux.
//! On Windows, interface 0 needs a WinUSB driver. nusb also supports macOS.
//!
//! # Hardware coverage
//!
//! Hardware testing used two connected Gen2 X1 adapters with firmware 2.2.4.10,
//! including legacy bus-off recovery. Other models and operating systems have
//! not been verified on hardware. XL models support CAN/CAN FD here, not CAN XL.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod config;
mod device;
mod error;
mod filter;
mod frame;
mod protocol;
mod transport;

pub use config::{ChannelConfig, FdMode, Mode, Termination};
pub use device::{devices, CanStatus, Device, DeviceInfo, Generation, OpenOptions};
pub use error::{Error, Result};
pub use filter::{Filter, ReceiveFilters};
pub use frame::{FdOptions, Frame, Id, ReceivedFrame, Timestamp};
