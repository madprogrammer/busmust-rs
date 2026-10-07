// SPDX-License-Identifier: GPL-2.0-or-later
// Rust adaptation and changes: 2026-10-07. See NOTICE.md for provenance.

use crate::{
    error::check_length,
    protocol::{self, Decoder},
    transport::{Transport, UsbTransport},
    ChannelConfig, Error, Frame, Mode, ReceivedFrame, Result, Termination,
};
use nusb::MaybeFuture;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

mod recovery;

const VENDOR_ID: u16 = 0x0810;

/// BUSMUST adapter hardware generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Generation {
    /// Second generation.
    Gen2,
    /// Second generation with revised hardware (X2R/X4R).
    Gen2_5,
    /// Third generation, including UTC timestamp tails.
    Gen3,
}

#[derive(Debug, Clone, Copy)]
struct Product {
    name: &'static str,
    channels: u8,
    generation: Generation,
}

fn product(pid: u16) -> Option<Product> {
    use Generation::*;
    let (name, channels, generation) = match pid {
        0xf012 => ("X1", 1, Gen2),
        0xf112 => ("X1 Pro", 1, Gen2),
        0xf122 => ("X2", 2, Gen2),
        0xf142 => ("X4", 4, Gen2),
        0xf182 => ("X8 Pi", 8, Gen2),
        0xe122 => ("X2R", 2, Gen2_5),
        0xe142 => ("X4R", 4, Gen2_5),
        0xf013 => ("X1", 1, Gen3),
        0xf023 => ("X2", 2, Gen3),
        0xf043 => ("X4", 4, Gen3),
        0x0043 => ("XL2", 2, Gen3),
        0x0083 => ("XL4", 4, Gen3),
        _ => return None,
    };
    Some(Product {
        name,
        channels,
        generation,
    })
}

/// Enumerate supported USB adapters without opening or configuring them.
///
/// Select explicitly by serial number or USB location when several are attached.
pub fn devices() -> Result<impl Iterator<Item = DeviceInfo>> {
    Ok(nusb::list_devices().wait()?.filter_map(|usb| {
        if usb.vendor_id() != VENDOR_ID {
            return None;
        }
        product(usb.product_id()).map(|product| DeviceInfo { usb, product })
    }))
}

/// Discovery information for one physical adapter, containing one or more channels.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    usb: nusb::DeviceInfo,
    product: Product,
}

impl DeviceInfo {
    /// Adapter model, such as `X2` or `X1 Pro`.
    pub fn model(&self) -> &'static str {
        self.product.name
    }
    /// Number of CAN channels; channel indices start at zero.
    pub fn channel_count(&self) -> u8 {
        self.product.channels
    }
    /// Adapter hardware generation.
    pub fn generation(&self) -> Generation {
        self.product.generation
    }
    /// USB product identifier.
    pub fn product_id(&self) -> u16 {
        self.usb.product_id()
    }
    /// USB vendor identifier (`0x0810`).
    pub fn vendor_id(&self) -> u16 {
        self.usb.vendor_id()
    }
    /// USB serial descriptor, if available from enumeration.
    pub fn serial_number(&self) -> Option<&str> {
        self.usb.serial_number()
    }
    /// Platform-specific USB bus identifier.
    pub fn bus_id(&self) -> &str {
        self.usb.bus_id()
    }
    /// USB device address on its bus. May change after reconnecting.
    pub fn address(&self) -> u8 {
        self.usb.device_address()
    }
    /// Claim USB interface 0. Channels remain inactive until configured.
    pub fn open(&self) -> Result<Device> {
        self.open_with_options(OpenOptions::default())
    }
    /// Claim USB interface 0 with explicit kernel-driver handling.
    pub fn open_with_options(&self, options: OpenOptions) -> Result<Device> {
        let transport = UsbTransport::open(&self.usb, options)?;
        let mut connection = Connection::new(transport, self.product);
        // Older devices may not implement this optional request. Other failures
        // (disconnect, malformed response) must not be silently swallowed.
        match connection.read_control(protocol::GET_VERSION, 0, 4) {
            Ok(bytes) => connection.firmware = Some(bytes.try_into().unwrap()),
            Err(Error::Transfer(nusb::transfer::TransferError::Stall)) => {}
            Err(error) => return Err(error),
        }
        Ok(Device {
            info: self.clone(),
            connection,
        })
    }
}

/// Options for claiming the physical USB interface.
#[derive(Debug, Default, Clone, Copy)]
pub struct OpenOptions {
    /// On Linux, detach a bound kernel driver and restore it on release.
    /// Defaults to false. On other platforms this has no effect.
    pub detach_kernel_driver: bool,
}

/// CAN controller health as reported by the adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanStatus {
    /// Controller is disconnected from the bus due to errors.
    pub bus_off: bool,
    /// Transmit controller is error-passive.
    pub tx_passive: bool,
    /// Receive controller is error-passive.
    pub rx_passive: bool,
    /// Transmit warning threshold exceeded.
    pub tx_warning: bool,
    /// Receive warning threshold exceeded.
    pub rx_warning: bool,
    /// Transmit error counter.
    pub tx_error_counter: u8,
    /// Receive error counter.
    pub rx_error_counter: u8,
}

/// Exclusive, blocking connection to a physical adapter and all its CAN channels.
///
/// Receive regularly to keep up with traffic. There is no background dispatch
/// thread: [`Self::receive`] returns frames from every configured channel in USB
/// order. Dropping this value stops configured channels and releases the USB
/// interface. Use [`Self::close`] to observe shutdown errors.
pub struct Device {
    info: DeviceInfo,
    connection: Connection<UsbTransport>,
}

impl Device {
    /// Discovery information for this adapter.
    pub fn info(&self) -> &DeviceInfo {
        &self.info
    }
    /// Firmware version as `[major, minor, patch, build]`, when supported.
    pub fn firmware_version(&self) -> Option<[u8; 4]> {
        self.connection.firmware
    }
    /// Configure and activate a zero-based CAN channel.
    ///
    /// Reconfiguration briefly stops that channel. A failed configuration leaves
    /// it inactive and attempts to stop it. Other channels retain their settings.
    /// A controller already in bus-off is recovered before configuration succeeds.
    pub fn configure_channel(&mut self, channel: u8, config: ChannelConfig) -> Result<()> {
        self.connection.configure(channel, config)
    }
    /// Last successfully applied configuration, or `None` for an inactive channel.
    pub fn channel_config(&self, channel: u8) -> Result<Option<&ChannelConfig>> {
        self.connection.validate_channel(channel)?;
        Ok(self.connection.channels[usize::from(channel)].as_ref())
    }
    /// Stop a channel. Calling this again after a successful stop has no effect.
    pub fn stop_channel(&mut self, channel: u8) -> Result<()> {
        self.connection.stop(channel)
    }
    /// Query flags and error counters without changing controller mode.
    pub fn status(&mut self, channel: u8) -> Result<CanStatus> {
        self.connection.status(channel)
    }
    /// Submit a frame to USB. Success does not imply a CAN acknowledgement.
    ///
    /// `timeout` bounds the bulk transfer; a preceding status request has its own
    /// two-second timeout. A zero duration is rejected. Timed-out or short writes
    /// are never retried: the frame may already have reached the adapter.
    /// Reopen after a failed write because its stream framing may be incomplete.
    pub fn send(&mut self, channel: u8, frame: &Frame, timeout: Duration) -> Result<()> {
        self.connection.send(channel, frame, timeout)
    }
    /// Receive from any configured channel, returning `None` on timeout.
    ///
    /// Zero duration polls queued frames and one completed USB transfer without
    /// waiting. Partial envelopes survive timeouts. USB/protocol errors fail the
    /// receive stream until the device is reopened. Timestamps remain in their
    /// original clock domains; no estimated host wall-clock conversion is made.
    pub fn receive(&mut self, timeout: Duration) -> Result<Option<ReceivedFrame>> {
        self.connection.receive(timeout)
    }
    /// Recover a configured bus-off controller and verify that it is healthy.
    ///
    /// Uses firmware recovery when available, otherwise the legacy Gen2 sequence:
    /// 1/8 Mbit/s internal loopback, 256 dummy remote frames, then restore settings.
    /// Recovery temporarily interrupts this channel and discards its buffered
    /// reception, while preserving sibling-channel frames. Up to eight attempts
    /// are made. Failures deactivate the channel; configure it again after fixing
    /// the cause. Application frames are never automatically retransmitted.
    pub fn recover_bus_off(&mut self, channel: u8) -> Result<CanStatus> {
        self.connection.recover(channel)
    }
    /// Stop every touched channel and release USB ownership, even on error.
    /// Returns the first shutdown error after attempting all channels.
    pub fn close(mut self) -> Result<()> {
        self.connection.shutdown()
    }
}

struct Connection<T: Transport> {
    transport: T,
    product: Product,
    firmware: Option<[u8; 4]>,
    channels: [Option<ChannelConfig>; 8],
    touched: [bool; 8],
    decoder: Decoder,
    received: VecDeque<ReceivedFrame>,
    receive_failed: bool,
    transmit_failed: bool,
}

impl<T: Transport> Connection<T> {
    fn new(transport: T, product: Product) -> Self {
        Self {
            transport,
            product,
            firmware: None,
            channels: [None; 8],
            touched: [false; 8],
            decoder: Decoder::default(),
            received: VecDeque::new(),
            receive_failed: false,
            transmit_failed: false,
        }
    }
    fn validate_channel(&self, channel: u8) -> Result<()> {
        if channel >= self.product.channels {
            Err(Error::InvalidChannel(channel))
        } else {
            Ok(())
        }
    }
    fn config(&self, channel: u8) -> Result<ChannelConfig> {
        self.validate_channel(channel)?;
        self.channels[usize::from(channel)].ok_or(Error::ChannelInactive(channel))
    }
    fn read_control(&mut self, request: u8, channel: u8, length: u16) -> Result<Vec<u8>> {
        let bytes = self.transport.control_in(request, channel, length)?;
        check_length(bytes.len(), usize::from(length))?;
        Ok(bytes)
    }
    fn configure(&mut self, channel: u8, config: ChannelConfig) -> Result<()> {
        self.validate_channel(channel)?;
        config.validate()?;
        let index = usize::from(channel);
        self.channels[index] = None;
        self.touched[index] = true;
        self.received.retain(|f| f.channel != channel);
        let result = self.apply_config(channel, config);
        match result {
            Ok(()) => {
                self.channels[index] = Some(config);
                Ok(())
            }
            Err(error) => {
                let _ = self.stop(channel);
                Err(error)
            }
        }
    }
    fn apply_config(&mut self, channel: u8, config: ChannelConfig) -> Result<()> {
        self.transport.control_out(
            protocol::SET_MODE,
            protocol::CONFIGURATION_MODE,
            channel,
            &[],
        )?;
        self.transport
            .control_out(protocol::SET_BITRATE, 0, channel, &config.bitrate_payload())?;
        self.transport.settle();
        for (slot, filter) in config.receive_filters.encode().iter().enumerate() {
            self.transport
                .control_out(protocol::SET_FILTER, slot as u16, channel, filter)?;
        }
        let termination = match config.termination {
            Termination::Unchanged => None,
            Termination::Disabled => Some(0),
            Termination::Ohms120 => Some(120),
        };
        if let Some(value) = termination {
            self.transport
                .control_out(protocol::SET_TERMINATION, value, channel, &[])?;
        }
        self.transport
            .control_out(protocol::SET_MODE, config.wire_mode(), channel, &[])?;
        self.transport.settle();
        if self.status(channel)?.bus_off {
            self.recover_configured(channel, config)?;
        }
        Ok(())
    }
    fn stop(&mut self, channel: u8) -> Result<()> {
        self.validate_channel(channel)?;
        let index = usize::from(channel);
        self.channels[index] = None;
        self.received.retain(|f| f.channel != channel);
        if self.touched[index] {
            self.transport.control_out(
                protocol::SET_MODE,
                protocol::CONFIGURATION_MODE,
                channel,
                &[],
            )?;
            self.touched[index] = false;
        }
        Ok(())
    }
    fn status(&mut self, channel: u8) -> Result<CanStatus> {
        self.validate_channel(channel)?;
        let bytes = self.read_control(protocol::GET_STATUS, channel, 8)?;
        Ok(CanStatus {
            bus_off: bytes[0] != 0,
            tx_passive: bytes[2] != 0,
            rx_passive: bytes[3] != 0,
            tx_warning: bytes[4] != 0,
            rx_warning: bytes[5] != 0,
            tx_error_counter: bytes[6],
            rx_error_counter: bytes[7],
        })
    }
    fn send(&mut self, channel: u8, frame: &Frame, timeout: Duration) -> Result<()> {
        let config = self.config(channel)?;
        if timeout.is_zero() {
            return Err(Error::InvalidArgument("send timeout must be nonzero"));
        }
        if config.mode == Mode::ListenOnly {
            return Err(Error::ListenOnly);
        }
        if frame.is_fd() && config.fd.is_none() {
            return Err(Error::InvalidArgument(
                "CAN FD frame requires a CAN FD channel",
            ));
        }
        if self.transmit_failed {
            return Err(Error::TransmitFailed);
        }
        if self.status(channel)?.bus_off {
            return Err(Error::BusOff(channel));
        }
        let result = self.transport.write(
            &protocol::encode(channel, frame, config.receive_own_messages),
            timeout,
        );
        if result.is_err() {
            self.transmit_failed = true;
        }
        result
    }
    fn pop_received(&mut self) -> Option<ReceivedFrame> {
        while let Some(mut frame) = self.received.pop_front() {
            let Some(config) = self
                .channels
                .get(usize::from(frame.channel))
                .and_then(Option::as_ref)
            else {
                continue;
            };
            if (frame.is_echo && !config.receive_own_messages)
                || !config.receive_filters.matches(&frame.frame)
            {
                continue;
            }
            if self.product.generation != Generation::Gen3 {
                frame.timestamp.unix_micros = None;
            }
            return Some(frame);
        }
        None
    }
    fn receive(&mut self, timeout: Duration) -> Result<Option<ReceivedFrame>> {
        if self.receive_failed {
            return Err(Error::ReceiveFailed);
        }
        if let Some(frame) = self.pop_received() {
            return Ok(Some(frame));
        }
        let start = Instant::now();
        loop {
            let bytes = match self.transport.read(timeout.saturating_sub(start.elapsed())) {
                Ok(Some(bytes)) => bytes,
                Ok(None) => return Ok(None),
                Err(error) => {
                    self.receive_failed = true;
                    return Err(error);
                }
            };
            match self.decoder.feed(&bytes) {
                Ok(frames) => self.received.extend(frames),
                Err(error) => {
                    self.receive_failed = true;
                    return Err(error);
                }
            }
            if let Some(frame) = self.pop_received() {
                return Ok(Some(frame));
            }
            if start.elapsed() >= timeout {
                return Ok(None);
            }
        }
    }
    fn recover(&mut self, channel: u8) -> Result<CanStatus> {
        self.validate_channel(channel)?;
        let status = self.status(channel)?;
        if !status.bus_off {
            return Ok(status);
        }
        let config = self.config(channel)?;
        match self.recover_configured(channel, config) {
            Ok(status) => Ok(status),
            Err(error) => {
                let _ = self.stop(channel);
                Err(error)
            }
        }
    }
    fn shutdown(&mut self) -> Result<()> {
        let mut first_error = None;
        for channel in 0..self.product.channels {
            if let Err(error) = self.stop(channel) {
                first_error.get_or_insert(error);
            }
        }
        first_error.map_or(Ok(()), Err)
    }
}

impl<T: Transport> Drop for Connection<T> {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

#[cfg(test)]
mod tests;
