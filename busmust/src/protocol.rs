//! Private, endian-explicit BUSMUST wire codec. No C layouts or casts.
use crate::frame::LENGTHS;
use crate::{Error, FdOptions, Frame, Id, ReceivedFrame, Result, Timestamp};

pub(crate) const SET_MODE: u8 = 0xc0;
pub(crate) const SET_BITRATE: u8 = 0xc2;
pub(crate) const SET_TERMINATION: u8 = 0xc3;
pub(crate) const SET_FILTER: u8 = 0xc8;
pub(crate) const GET_STATUS: u8 = 0xd1;
pub(crate) const GET_VERSION: u8 = 0xf1;
pub(crate) const RECOVER_BUS_OFF: u8 = 0xf5;
pub(crate) const CONFIGURATION_MODE: u16 = 4;
const IDE: u32 = 0x10;
const RTR: u32 = 0x20;
const BRS: u32 = 0x40;
const FDF: u32 = 0x80;
const ESI: u32 = 0x100;
const ECHO: u32 = 0x20000;
const HEADER_LEN: usize = 8;
const MAX_PAYLOAD: usize = 1024;

fn align4(n: usize) -> usize {
    (n + 3) & !3
}

pub(crate) fn encode(channel: u8, frame: &Frame, echo: bool) -> Vec<u8> {
    let mut packet = vec![0; 16 + align4(frame.len())];
    let header = 0xf002 | (u16::from(channel) << 8);
    let length = (packet.len() - HEADER_LEN) as u16;
    packet[..2].copy_from_slice(&header.to_le_bytes());
    packet[2..4].copy_from_slice(&length.to_le_bytes());
    let id = frame.id();
    let raw = if id.is_extended() {
        ((id.as_raw() & 0x3ffff) << 11) | (id.as_raw() >> 18)
    } else {
        id.as_raw()
    };
    packet[8..12].copy_from_slice(&raw.to_le_bytes());
    let mut control = u32::from(frame.dlc());
    if id.is_extended() {
        control |= IDE;
    }
    if frame.is_remote() {
        control |= RTR;
    }
    if echo {
        control |= ECHO;
    }
    if let Some(flags) = frame.fd_options() {
        control |= FDF;
        if flags.bitrate_switch {
            control |= BRS;
        }
    }
    packet[12..16].copy_from_slice(&control.to_le_bytes());
    packet[16..16 + frame.data().len()].copy_from_slice(frame.data());
    packet
}

/// Retains only the unfinished envelope between transfers.
#[derive(Default)]
pub(crate) struct Decoder {
    buffer: Vec<u8>,
    discard_partial_channel: Option<u8>,
}

impl Decoder {
    pub(crate) fn feed(&mut self, bytes: &[u8]) -> Result<Vec<ReceivedFrame>> {
        self.buffer.extend_from_slice(bytes);
        let result = self.decode();
        if result.is_err() {
            self.buffer.clear();
            self.discard_partial_channel = None;
        }
        result
    }
    // Recovery may stop after the beginning of an envelope. Remember to discard
    // that envelope if it belongs to the recovering channel when it completes.
    pub(crate) fn discard_pending_from(&mut self, channel: u8) {
        if !self.buffer.is_empty() {
            self.discard_partial_channel = Some(channel);
        }
    }
    fn decode(&mut self) -> Result<Vec<ReceivedFrame>> {
        let mut frames = Vec::new();
        let mut offset = 0;
        while self.buffer.len() - offset >= HEADER_LEN {
            let envelope = &self.buffer[offset..];
            let header = u16::from_le_bytes([envelope[0], envelope[1]]);
            let length = usize::from(u16::from_le_bytes([envelope[2], envelope[3]]));
            if length > MAX_PAYLOAD {
                return Err(Error::Protocol("payload exceeds 1024 bytes"));
            }
            let total = align4(HEADER_LEN + length);
            if envelope.len() < total {
                break;
            }
            if matches!(header & 0xf, 2 | 10) && header & 0xe0 == 0 {
                let frame = decode_can(header, u32_at(envelope, 4), &envelope[8..8 + length])?;
                if offset != 0 || self.discard_partial_channel != Some(frame.channel) {
                    frames.push(frame);
                }
            }
            offset += total;
        }
        if offset != 0 {
            self.discard_partial_channel = None;
        }
        self.buffer.drain(..offset);
        Ok(frames)
    }
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn decode_can(header: u16, ticks: u32, payload: &[u8]) -> Result<ReceivedFrame> {
    let tail_len = if header & 0x10 != 0 { 16 } else { 0 };
    if payload.len() < 8 + tail_len {
        return Err(Error::Protocol(
            "missing CAN control words or timestamp tail",
        ));
    }
    let raw_id = u32_at(payload, 0);
    let control = u32_at(payload, 4);
    let dlc = (control & 0xf) as usize;
    let remote = control & RTR != 0;
    // Older firmware sometimes omits FDF for DLC > 8.
    let fd = control & FDF != 0 || dlc > 8;
    if remote && fd {
        return Err(Error::Protocol("remote CAN FD frame"));
    }
    if !fd && control & (BRS | ESI) != 0 {
        return Err(Error::Protocol("CAN FD flags on classic frame"));
    }
    let size = usize::from(LENGTHS[dlc]);
    let data_len = if remote { 0 } else { size };
    if payload.len() - tail_len < 8 + data_len {
        return Err(Error::Protocol("DLC exceeds CAN payload"));
    }
    let id = if control & IDE != 0 {
        Id::extended(((raw_id & 0x7ff) << 18) | ((raw_id >> 11) & 0x3ffff))?
    } else {
        Id::standard((raw_id & 0x7ff) as u16)?
    };
    let data = &payload[8..8 + data_len];
    let mut frame = if remote {
        Frame::remote(id, size as u8)?
    } else if fd {
        Frame::fd(
            id,
            data,
            FdOptions {
                bitrate_switch: control & BRS != 0,
            },
        )?
    } else {
        Frame::classic(id, data)?
    };
    frame.error_state_indicator = control & ESI != 0;
    let unix_micros = if tail_len != 0 {
        let tail = align4(8 + data_len);
        if payload.len() < tail + 16 {
            return Err(Error::Protocol("unaligned or truncated timestamp tail"));
        }
        let value = u64::from_le_bytes(payload[tail + 8..tail + 16].try_into().unwrap());
        (value != 0).then_some(value)
    } else {
        None
    };
    let dest = ((header >> 8) & 0xf) as u8;
    let channel = if dest == 0xf {
        (header >> 12) as u8
    } else {
        dest
    };
    Ok(ReceivedFrame {
        channel,
        frame,
        timestamp: Timestamp {
            device_micros: ticks,
            unix_micros,
        },
        is_echo: header & 0xf == 10,
    })
}

#[cfg(test)]
mod tests;
