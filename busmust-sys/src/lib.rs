#![allow(non_upper_case_globals, non_snake_case)]

extern crate bitfield_struct;

#[macro_use]
extern crate bitflags;
extern crate core;

mod types;
mod api;
mod bitrate_builder;
mod can_message_builder;
mod data_builder;

pub use types::*;
pub use api::*;

impl BMCanMessage {
    pub fn payload(&self) -> &[u8] {
        unsafe {
            &self.payload[0..self.ctrl.rx.dlc() as usize]
        }
    }

    pub fn sid(&self) -> u16 {
        self.mid.sid()
    }

    pub fn eid(&self) -> u32 {
        self.mid.eid()
    }

    fn to_bytes(&self) -> Vec<u8> {
        let ptr = self as *const BMCanMessage as *const u8;
        let len = std::mem::size_of::<BMCanMessage>();
        unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec()
    }
}