//! Enumerate adapters. Pass a USB serial number to run an internal loopback test.
use busmust::{ChannelConfig, Frame, Id, Mode};
use std::{error::Error, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    let serial = std::env::args().nth(1);
    let devices: Vec<_> = busmust::devices()?.collect();
    for info in &devices {
        println!(
            "{} {:?}, serial={}, USB {}:{}, {} channel(s)",
            info.model(),
            info.generation(),
            info.serial_number().unwrap_or("<unavailable>"),
            info.bus_id(),
            info.address(),
            info.channel_count()
        );
    }
    let Some(serial) = serial else {
        return Ok(());
    };
    let mut matching = devices
        .iter()
        .filter(|d| d.serial_number() == Some(serial.as_str()));
    let info = matching
        .next()
        .ok_or("no adapter with that serial number")?;
    if matching.next().is_some() {
        return Err("serial number is ambiguous".into());
    }
    let mut device = info.open()?;
    println!(
        "Firmware: {:?}; initial status: {:?}",
        device.firmware_version(),
        device.status(0)?
    );
    device.configure_channel(
        0,
        ChannelConfig {
            mode: Mode::InternalLoopback,
            ..Default::default()
        },
    )?;
    let frame = Frame::classic(Id::standard(0x123)?, &[1, 2, 3, 4, 5, 6, 7, 8])?;
    device.send(0, &frame, Duration::from_secs(1))?;
    let received = device
        .receive(Duration::from_secs(1))?
        .ok_or("loopback timed out")?;
    if received.channel != 0 || received.frame != frame {
        return Err("loopback returned an unexpected frame".into());
    }
    println!("Loopback passed: {received:?}");
    device.close()?;
    Ok(())
}
