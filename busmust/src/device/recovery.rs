use super::*;
use crate::{FdMode, Id};

const ATTEMPTS: usize = 8;
const DUMMY_COUNT: usize = 256;
const MAX_DRAIN_TRANSFERS: usize = 64;
const MAX_BUFFERED_FRAMES: usize = 4096;

impl<T: Transport> Connection<T> {
    pub(super) fn recover_configured(
        &mut self,
        channel: u8,
        config: ChannelConfig,
    ) -> Result<CanStatus> {
        if self.transmit_failed {
            return Err(Error::TransmitFailed);
        }
        if self.receive_failed {
            return Err(Error::ReceiveFailed);
        }
        let minimum = if self.product.generation == Generation::Gen3 {
            [3, 1, 0, 0]
        } else {
            [2, 6, 0, 0]
        };
        let firmware_recovery = self.firmware.is_some_and(|version| version >= minimum);
        for attempt in 0..ATTEMPTS {
            if firmware_recovery {
                match self.read_control(protocol::RECOVER_BUS_OFF, channel, 0) {
                    Ok(_) => {
                        self.transport.delay(Duration::from_millis(50));
                        let status = self.status(channel)?;
                        if !status.bus_off {
                            return Ok(status);
                        }
                    }
                    // Some firmware reports a version that predates implementation
                    // of this request; a stalled request can use legacy recovery.
                    Err(Error::Transfer(nusb::transfer::TransferError::Stall)) => {}
                    Err(error) => return Err(error),
                }
            }
            self.legacy_recovery(channel, config)?;
            let status = self.status(channel)?;
            if !status.bus_off {
                return Ok(status);
            }
            if attempt + 1 < ATTEMPTS {
                self.transport.delay(Duration::from_millis(100));
            }
        }
        Err(Error::BusOff(channel))
    }

    fn legacy_recovery(&mut self, channel: u8, config: ChannelConfig) -> Result<()> {
        self.received.retain(|frame| frame.channel != channel);
        let result = self.loopback_dummies(channel);
        // Always attempt to restore the requested timing, even when entering
        // loopback, writing dummies, or draining reception failed. On failure,
        // remain in configuration mode instead of reactivating an uncertain bus.
        let mode = if result.is_ok() {
            config.wire_mode()
        } else {
            protocol::CONFIGURATION_MODE
        };
        let restored = self.restore_timing(channel, config, mode);
        result.and(restored)
    }

    fn loopback_dummies(&mut self, channel: u8) -> Result<()> {
        let temporary = ChannelConfig {
            bitrate: 1_000_000,
            data_bitrate: 8_000_000,
            sample_point: 87,
            data_sample_point: 75,
            fd: Some(FdMode::Iso),
            ..Default::default()
        };
        self.restore_timing(channel, temporary, 2)?;
        let dummy = Frame::remote(Id::standard(0)?, 0)?;
        let packet = protocol::encode(channel, &dummy, false).repeat(DUMMY_COUNT);
        if let Err(error) = self.transport.write(&packet, Duration::from_secs(1)) {
            self.transmit_failed = true;
            return Err(error);
        }
        self.transport.delay(Duration::from_millis(50));
        self.transport.control_out(
            protocol::SET_MODE,
            protocol::CONFIGURATION_MODE,
            channel,
            &[],
        )?;
        self.transport.settle();
        let result = self.drain_recovery(channel);
        if result.is_err() {
            self.receive_failed = true;
        }
        result
    }

    fn restore_timing(&mut self, channel: u8, config: ChannelConfig, mode: u16) -> Result<()> {
        self.transport.control_out(
            protocol::SET_MODE,
            protocol::CONFIGURATION_MODE,
            channel,
            &[],
        )?;
        self.transport
            .control_out(protocol::SET_BITRATE, 0, channel, &config.bitrate_payload())?;
        self.transport.settle();
        self.transport
            .control_out(protocol::SET_MODE, mode, channel, &[])?;
        self.transport.settle();
        Ok(())
    }

    fn drain_recovery(&mut self, channel: u8) -> Result<()> {
        for _ in 0..MAX_DRAIN_TRANSFERS {
            let bytes = match self.transport.read(Duration::from_millis(10))? {
                Some(bytes) if bytes.is_empty() => continue,
                Some(bytes) => bytes,
                // A USB read can contain data without completing when the last
                // packet was full-sized. Cancel and collect those bytes too.
                None => self.transport.flush_read()?,
            };
            if bytes.is_empty() {
                self.decoder.discard_pending_from(channel);
                return Ok(());
            }
            for frame in self.decoder.feed(&bytes)? {
                if frame.channel != channel
                    && self
                        .channels
                        .get(usize::from(frame.channel))
                        .is_some_and(Option::is_some)
                {
                    if self.received.len() >= MAX_BUFFERED_FRAMES {
                        return Err(Error::RecoveryFailed(
                            "sibling-channel receive queue filled",
                        ));
                    }
                    self.received.push_back(frame);
                }
            }
        }
        Err(Error::RecoveryFailed(
            "receive stream did not drain after stopping loopback",
        ))
    }
}
