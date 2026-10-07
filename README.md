# BUSMUST CAN for Rust

[![CI](https://github.com/madprogrammer/busmust-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/madprogrammer/busmust-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/busmust.svg)](https://crates.io/crates/busmust)
[![Documentation](https://docs.rs/busmust/badge.svg)](https://docs.rs/busmust)

A native Rust driver for BUSMUST USB CAN and CAN FD adapters, using
[nusb](https://docs.rs/nusb/0.2.7/nusb/). No vendor SDK, C bindings, libusb,
or asynchronous runtime is required. The crate forbids unsafe Rust.

```toml
[dependencies]
busmust = "0.2.1"
```

```rust,no_run
use busmust::{ChannelConfig, Frame, Id};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let info = busmust::devices()?
        .find(|device| device.serial_number() == Some("YOUR_SERIAL"))
        .ok_or("adapter not found")?;
    let mut device = info.open()?;
    device.configure_channel(0, ChannelConfig::default())?;

    let frame = Frame::classic(Id::standard(0x123)?, &[1, 2, 3, 4])?;
    device.send(0, &frame, Duration::from_secs(1))?;
    if let Some(received) = device.receive(Duration::from_secs(1))? {
        println!("channel {}: {:?}", received.channel, received.frame);
    }
    device.close()?;
    Ok(())
}
```

See the [API documentation](https://docs.rs/busmust) for configuration, filters,
recovery, and platform setup. Requires Rust 1.85 or newer.

Version 0.2 intentionally replaces the SDK-shaped 0.1 API:

| Previous API | Replacement |
| --- | --- |
| `dmgr::initialize()` / `terminate()` | No global initialization |
| `dmgr::enum_devices()` | `busmust::devices()`; one entry per physical adapter |
| `Device::open_ex()` and separate setting calls | `DeviceInfo::open()`, then `configure_channel()` |
| `busmust_sys::BMCanMessage` / `BMData` builders | Validated `Id`, `Frame`, and `FdOptions` |
| Integer millisecond timeouts | `std::time::Duration` |
| Read plus notification handle | `receive(timeout) -> Result<Option<ReceivedFrame>>` |
| SDK status codes / text lookup | Structured `Error` and `CanStatus` |
| Manual handle cleanup | Ownership and `Drop`; optional fallible `close()` |

Run an internal loopback test on a selected adapter:

```sh
cargo run --example demo -- YOUR_SERIAL
```

## License

[GPL-2.0-or-later](LICENSE-GPL). The native driver derives from
`python-can-busmust` and `bmsocketcan`; see [attributions](NOTICE.md).
