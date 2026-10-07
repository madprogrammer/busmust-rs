// SPDX-License-Identifier: GPL-2.0-or-later
// Portions derived from bmsocketcan: Copyright (C) 2026 Busmust Tech Co.,Ltd
// Rust adaptation and changes: 2026-10-07. See NOTICE.md for provenance.

use crate::{error::check_length, Error, OpenOptions, Result};
use nusb::{
    transfer::{
        Buffer, Bulk, ControlIn, ControlOut, ControlType, In, Out, Recipient, TransferError,
    },
    Endpoint, Interface, MaybeFuture,
};
use std::time::Duration;

pub(crate) const CONTROL_TIMEOUT: Duration = Duration::from_secs(2);
const READ_SIZE: usize = 16 * 1024;

/// Internal seam for testing hardware operations without a connected adapter.
pub(crate) trait Transport {
    fn control_in(&mut self, request: u8, channel: u8, length: u16) -> Result<Vec<u8>>;
    fn control_out(&mut self, request: u8, value: u16, channel: u8, data: &[u8]) -> Result<()>;
    fn write(&mut self, data: &[u8], timeout: Duration) -> Result<()>;
    fn read(&mut self, timeout: Duration) -> Result<Option<Vec<u8>>>;
    // Finish the pending read without throwing away partially transferred bytes.
    fn flush_read(&mut self) -> Result<Vec<u8>>;
    fn delay(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }
    fn settle(&mut self) {
        self.delay(Duration::from_millis(10));
    }
}

pub(crate) struct UsbTransport {
    interface: Interface,
    input: Endpoint<Bulk, In>,
    output: Endpoint<Bulk, Out>,
}

impl UsbTransport {
    pub(crate) fn open(info: &nusb::DeviceInfo, options: OpenOptions) -> Result<Self> {
        let device = info.open().wait()?;
        let interface = if options.detach_kernel_driver {
            device.detach_and_claim_interface(0).wait()?
        } else {
            device.claim_interface(0).wait()?
        };
        let descriptor = interface
            .descriptor()
            .ok_or(Error::Protocol("missing USB interface descriptor"))?;
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        for ep in descriptor.endpoints() {
            if ep.transfer_type() == nusb::descriptors::TransferType::Bulk {
                if ep.address() & 0x80 != 0 {
                    inputs.push(ep.address());
                } else {
                    outputs.push(ep.address());
                }
            }
        }
        if inputs.len() != 1 || outputs.len() != 1 {
            return Err(Error::Protocol(
                "expected one bulk IN and one bulk OUT endpoint on interface 0",
            ));
        }
        let mut input = interface.endpoint::<Bulk, In>(inputs[0])?;
        let output = interface.endpoint::<Bulk, Out>(outputs[0])?;
        // Keep one read pending across receive timeouts. Cancelling on every
        // timeout can discard partial USB data and break envelope framing.
        input.submit(Buffer::new(READ_SIZE));
        Ok(Self {
            interface,
            input,
            output,
        })
    }
}

impl Transport for UsbTransport {
    fn control_in(&mut self, request: u8, channel: u8, length: u16) -> Result<Vec<u8>> {
        let data = self
            .interface
            .control_in(
                ControlIn {
                    control_type: ControlType::Vendor,
                    recipient: Recipient::Device,
                    request,
                    value: 0,
                    index: u16::from(channel),
                    length,
                },
                CONTROL_TIMEOUT,
            )
            .wait()?;
        check_length(data.len(), usize::from(length))?;
        Ok(data)
    }
    fn control_out(&mut self, request: u8, value: u16, channel: u8, data: &[u8]) -> Result<()> {
        self.interface
            .control_out(
                ControlOut {
                    control_type: ControlType::Vendor,
                    recipient: Recipient::Device,
                    request,
                    value,
                    index: u16::from(channel),
                    data,
                },
                CONTROL_TIMEOUT,
            )
            .wait()?;
        Ok(())
    }
    fn write(&mut self, data: &[u8], timeout: Duration) -> Result<()> {
        if timeout.is_zero() {
            return Err(Error::Timeout);
        }
        let completion = self.output.transfer_blocking(data.to_vec().into(), timeout);
        if completion.status == Err(TransferError::Cancelled) {
            return Err(Error::Timeout);
        }
        completion.status?;
        check_length(completion.actual_len, data.len())
    }
    fn flush_read(&mut self) -> Result<Vec<u8>> {
        self.finish_pending_read()
    }
    fn read(&mut self, timeout: Duration) -> Result<Option<Vec<u8>>> {
        let Some(completion) = self.input.wait_next_complete(timeout) else {
            return Ok(None);
        };
        let data = completion.into_result()?;
        let bytes = data.to_vec();
        self.input.submit(Buffer::new(READ_SIZE));
        Ok(Some(bytes))
    }
}

impl UsbTransport {
    fn finish_pending_read(&mut self) -> Result<Vec<u8>> {
        self.input.cancel_all();
        let completion = loop {
            if let Some(completion) = self.input.wait_next_complete(Duration::from_secs(1)) {
                break completion;
            }
        };
        match completion.status {
            Ok(()) | Err(TransferError::Cancelled) => {}
            Err(error) => return Err(error.into()),
        }
        let bytes = completion.buffer.to_vec();
        self.input.submit(Buffer::new(READ_SIZE));
        Ok(bytes)
    }
}
