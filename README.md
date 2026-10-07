# BUSMUST CAN for Rust

[![CI](https://github.com/madprogrammer/busmust-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/madprogrammer/busmust-rs/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/busmust.svg)](https://crates.io/crates/busmust)
[![Documentation](https://docs.rs/busmust/badge.svg)](https://docs.rs/busmust)

A native Rust driver for BUSMUST USB CAN and CAN FD adapters, using
[nusb](https://docs.rs/nusb/0.2.7/nusb/). No vendor SDK, C bindings, libusb,
or asynchronous runtime is required. The crate forbids unsafe Rust.

```toml
[dependencies]
busmust = "0.2"
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

One `Device` owns a physical adapter and all its channels. Configure each desired
channel once, then use `send(channel, ...)` and `receive(...)`. Receive returns
frames from all configured channels in USB order, with their channel number.
There are no per-channel USB readers competing for the same endpoint and no
background threads. Call `receive` regularly to keep up with bus traffic. Share a
device through application-owned synchronization if multiple threads need access.

Defaults are classic CAN at **500,000 bits/s**, normal mode, unchanged termination,
and no transmit echoes. Bitrates are positive multiples of 1,000 bits/s. Sample
points are whole percentages (1–99); the firmware wire format has no fractional
precision. Validation checks protocol ranges; the adapter may reject timings it
cannot produce.

For CAN FD, set `fd: Some(FdMode::Iso)` and `data_bitrate` on `ChannelConfig`, and
construct frames with `Frame::fd(id, data, FdOptions { ... })`. FD data is padded
with zeros to the next legal CAN FD length during construction; `frame.data()`
shows the exact bytes that will be sent. `Frame::remote(id, requested_len)` creates
a classic remote request with no payload. Identifier constructors enforce the
11-bit and 29-bit ranges. Listen-only, internal loopback, non-ISO FD, one-shot
transmission, hardware echoes, and 120-ohm termination are configurable.
`Frame::is_error_passive()` exposes received ESI; ESI is supplied by the transmitting
controller and is not a writable transmit option.

Set `ChannelConfig::receive_filters` to `ReceiveFilters::Match(Filter::exact(id))`
for an exact identifier, `Filter::new(id, mask)` for a range, or
`ReceiveFilters::Either(first, second)` for two alternatives. Standard and extended
formats are distinguished. Filters can additionally constrain remote/data, FD, and
BRS flags. The two hardware slots are programmed, and the same policy is checked
on receipt (including echoes): Gen2 firmware 2.2.4.10 ignores some basic-filter
flag constraints, so the Rust check supplies consistent behavior. Reconfiguring
with `ReceiveFilters::AcceptAll` removes both filters.

`send` confirms USB completion, **not a CAN acknowledgement**. It checks bus-off
before sending. Its nonzero `Duration` limits the bulk transfer; the preceding
status query has a separate two-second control timeout. USB cancellation may take
additional time to finish. A failed or short write is never retried automatically
and disables further sends until the device is reopened, since a partial envelope
may have reached the adapter. Bulk timeout is reported as `Error::Timeout`.

`receive` returns `Ok(None)` on timeout. `Duration::ZERO` polls the local queue and
one completed USB transfer. Fragmented envelopes survive timeouts; concatenated
frames are retained for subsequent calls. Protocol and USB receive errors disable
receiving until the device is reopened. Unconfigured channels and unrequested
transmit echoes are ignored. Reconfiguration can still expose frames buffered by
the adapter before the mode change.

Timestamps expose the original wrapping 32-bit microsecond device counter and,
when present on Gen3 adapters, UTC microseconds since the Unix epoch. The counter
wraps about every 71.6 minutes. The driver makes no host-clock or wrap-count
estimate. `status(channel)` provides bus-off flags, passive/warning flags, and
transmit/receive error counters.

`recover_bus_off(channel)` uses firmware recovery on Gen2/2.5 >= 2.6.0.0 and
Gen3 >= 3.1.0.0. Older or unknown firmware, and firmware recovery that does not
clear bus-off, use the legacy sequence needed by Gen2 X1 adapters: temporarily
configure 1/8 Mbit/s internal loopback, submit 256 zero-length remote frames, then
restore the requested bitrate and mode. Recovery retries at most eight times and
reports `Error::BusOff` if the controller remains stuck. Configuration performs
this recovery automatically when the controller starts bus-off; later runtime
recovery is explicit and never retransmits application frames.

Legacy recovery interrupts the affected channel and discards its pending reception.
Sibling-channel frames are retained while draining loopback traffic, including
bytes in partially completed USB transfers. Draining is bounded to 64 transfers
and 4,096 buffered sibling frames; exceeding either bound is an explicit recovery
error requiring a reopen. USB/protocol failures during recovery attempt to restore
timing and stop the affected channel; successful siblings keep their configuration.

Dropping `Device` attempts to stop all channels it configured, cancels pending
transfers, and releases USB ownership. `close(self)` also reports shutdown errors.
Channel configuration failures attempt to return that channel to configuration
mode, and shutdown continues to other channels if one stop fails.

Supported USB vendor ID: `0810`.

| Generation | Models (product IDs) |
| --- | --- |
| 2 | X1 (`f012`), X1 Pro (`f112`), X2 (`f122`), X4 (`f142`), X8 Pi (`f182`) |
| 2.5 | X2R (`e122`), X4R (`e142`) |
| 3 | X1 (`f013`), X2 (`f023`), X4 (`f043`), XL2 (`0043`), XL4 (`0083`) |

XL models are supported for CAN/CAN FD only, not CAN XL. Device discovery returns
serial number, USB location, model, generation, and channel count; it never picks
an arbitrary adapter on the application's behalf.

On Linux, grant your user access to the USB device, for example with an appropriate
udev rule matching vendor `0810`. If SocketCAN's `bmcan` driver owns interface 0,
release it or explicitly use:

```rust,ignore
let mut device = info.open_with_options(busmust::OpenOptions {
    detach_kernel_driver: true,
})?;
```

nusb restores the kernel driver when the interface is released on Linux. On
Windows, interface 0 needs a WinUSB driver. Linux, macOS, and Windows are supported
by nusb; this refactor has been compiled and tested on Linux with two connected
Gen2 X1 adapters running firmware 2.2.4.10. Other models, firmware versions, and
operating systems have not been verified on hardware.

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

The `busmust-sys` crate and its native build script have been removed. Vendor SDK
functions for ISO-TP, routing, logging, and scheduled transmission are not exposed
by this CAN frame API.

Run the checks with Rust 1.85 or newer:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
cargo doc --workspace --no-deps
```

The hardware-free tests cover wire vectors, FD lengths, identifiers, malformed and
fragmented input, routing, timestamps, configuration rollback, cleanup, recovery,
and transfer failures. `cargo run --example demo` only enumerates adapters. To run
an internal loopback check on an explicitly selected adapter:

```sh
cargo run --example demo -- YOUR_SERIAL
```

For a repeatable bidirectional sweep on two connected Gen2 X1 adapters:

```sh
cargo run --example sweep -- SERIAL_A SERIAL_B
# Optionally run just cases whose names contain a substring:
cargo run --example sweep -- SERIAL_A SERIAL_B filters
```

The sweep changes both adapters' configuration and exercises classic CAN from
10 kbit/s to 1 Mbit/s, ISO FD nominal/data bitrate combinations through 8 Mbit/s,
non-ISO FD, all FD payload lengths, standard/extended/remote frames, termination
combinations, sample points, filters, echoes, one-shot mode, listen-only rejection,
loopback isolation, receive order/timestamps, lifecycle errors, and deliberately
induced bus-off followed by legacy recovery. Each frame is checked for content and
flags; successful communication cases check controller error counters. Final
cleanup stops both channels and enables their 120-ohm termination resistors.

Results are written to `hardware-results.tsv` (or `hardware-results-focused.tsv`
for a selected subset). Passing traffic with disabled termination only describes
the attached cable/setup; the test does not electrically measure resistor values.
Gen3 UTC timestamps and sibling-channel USB dispatch are covered by codec/mock
tests, since X1 adapters have one channel each.

Protocol behavior was checked against the adjacent `python-can-busmust` project
(`src/busmust/protocol.py`, `transport.py`) and its `bmsocketcan` reference
(`inc/bm_usb_def.h`, `src/bmcan_proto.c`, `src/bmcan_usb.c`,
`src/bmcan_netdev.c`). The Rust codec uses
explicit byte encoding and decoding, without including vendor headers or linking
the vendor library.

CI runs tests on Linux, macOS, and Windows, checks the Rust 1.85 minimum version,
and validates formatting, Clippy, documentation, and the packaged crate. Hardware
checks remain manual: the 0.2.0 release passed 74 cases between two connected Gen2
X1 adapters, in addition to 41 software tests.

To release, update `busmust/Cargo.toml` and `Cargo.lock`, merge into `master`, and
wait for CI. Tag that commit as `v<version>` and publish a GitHub release. The
release workflow repeats CI on the tag, verifies its version, builds the package,
and attaches the `.crate` archive to the release. Publish the same commit to
crates.io locally with `cargo publish --package busmust --locked`, using a local
Cargo credential or a `CARGO_REGISTRY_TOKEN` environment variable. No crates.io
credential is stored in GitHub. Local token files and hardware reports are ignored
by Git and are not included in the published package.

Licensed under either [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
