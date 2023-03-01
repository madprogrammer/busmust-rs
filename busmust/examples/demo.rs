extern crate busmust;
extern crate busmust_sys;

use busmust::dmgr;
use busmust_sys::{BMBitrate, BMCanMessage, BMCanMode, BMData, BMLogLevel};

fn main() {
    dmgr::initialize().unwrap();

    for mut device in dmgr::enum_devices().unwrap() {
        println!("name: {}", device.name());
        println!("serial num: {}", device.serial_number());
        println!("unique id: {}", device.unique_id());
        println!("port: {}", device.port());
        println!("vid: {}", device.vendor_id());
        println!("pid: {}", device.product_id());
        println!("caps: {:?}", device.caps());

        device.set_log_level(BMLogLevel::Info);
        device.open_ex().unwrap();
        device.set_bitrate(BMBitrate::builder().bitrate(250).build()).unwrap();
        device.set_can_mode(BMCanMode::InternalLoopback).unwrap();

        for _ in 0..500 {
            let msg = BMCanMessage::builder()
                .sid(0x123)
                .payload(vec![1, 2, 3, 4, 5, 6, 7, 8])
                .build();

            device.write_can_message(msg, Some(100)).unwrap();
            device.wait_for_notification(Some(100));
            device.read_can_message().unwrap().expect("no message");
        }

        let msg = BMCanMessage::builder()
            .sid(0x123)
            .payload(vec![1, 2, 3, 4, 5, 6, 7, 8])
            .build();

        let data = BMData::builder()
            .can_message(msg)
            .build();

        device.write(data, Some(1000)).unwrap();
        device.clear_buffer().unwrap();
        device.close().unwrap();
    }

    dmgr::terminate().unwrap();
}
