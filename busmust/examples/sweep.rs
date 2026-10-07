//! Hardware verification for two directly connected adapters.
//! Usage: cargo run --example sweep -- SERIAL_A SERIAL_B [CASE_SUBSTRING]
//! Writes hardware-results.tsv and exits unsuccessfully if any case fails.
use busmust::{
    ChannelConfig, Device, DeviceInfo, Error, FdMode, FdOptions, Filter, Frame, Id, Mode,
    ReceiveFilters, Termination,
};
use std::{
    error::Error as StdError,
    fs::File,
    io::Write,
    time::{Duration, Instant},
};

type TestResult<T = ()> = Result<T, Box<dyn StdError>>;
const TIMEOUT: Duration = Duration::from_millis(500);

struct Sweep {
    a: DeviceInfo,
    b: DeviceInfo,
    selected: String,
    report: File,
    passed: usize,
    failed: usize,
}
impl Sweep {
    fn run(
        &mut self,
        name: &str,
        test: impl FnOnce(&mut Device, &mut Device) -> TestResult,
    ) -> TestResult {
        if !name.contains(&self.selected) && name != "final-reopen-and-cleanup" {
            return Ok(());
        }
        let start = Instant::now();
        let result = (|| -> TestResult {
            let mut a = self.a.open()?;
            let mut b = self.b.open()?;
            let result = test(&mut a, &mut b);
            let diagnostic = if result.is_err() {
                format!("; A={:?}; B={:?}", a.status(0), b.status(0))
            } else {
                String::new()
            };
            let close_a = a.close();
            let close_b = b.close();
            if let Err(error) = result {
                return Err(format!("{error}{diagnostic}").into());
            }
            close_a?;
            close_b?;
            Ok(())
        })();
        let (outcome, detail) = match result {
            Ok(()) => {
                self.passed += 1;
                ("PASS", String::new())
            }
            Err(error) => {
                self.failed += 1;
                ("FAIL", error.to_string().replace(['\t', '\n'], " "))
            }
        };
        println!("{outcome} {name} {detail}");
        writeln!(
            self.report,
            "{name}\t{outcome}\t{}\t{detail}",
            start.elapsed().as_millis()
        )?;
        self.report.flush()?;
        Ok(())
    }
}
fn config() -> ChannelConfig {
    ChannelConfig {
        termination: Termination::Ohms120,
        ..Default::default()
    }
}
fn configure(a: &mut Device, b: &mut Device, ca: ChannelConfig, cb: ChannelConfig) -> TestResult {
    a.stop_channel(0)?;
    b.stop_channel(0)?;
    a.configure_channel(0, ca)?;
    b.configure_channel(0, cb)?;
    drain(a)?;
    drain(b)?;
    Ok(())
}
fn drain(device: &mut Device) -> TestResult {
    for _ in 0..1024 {
        if device.receive(Duration::from_millis(5))?.is_none() {
            return Ok(());
        }
    }
    Err("receive stream did not become quiet".into())
}
fn healthy(device: &mut Device) -> TestResult {
    let status = device.status(0)?;
    if status.bus_off
        || status.tx_passive
        || status.rx_passive
        || status.tx_error_counter != 0
        || status.rx_error_counter != 0
    {
        return Err(format!("unhealthy controller: {status:?}").into());
    }
    Ok(())
}
fn transfer(sender: &mut Device, receiver: &mut Device, frame: &Frame) -> TestResult {
    sender.send(0, frame, TIMEOUT)?;
    let received = receiver
        .receive(TIMEOUT)?
        .ok_or("expected frame did not arrive")?;
    if received.channel != 0 || received.is_echo || received.frame != *frame {
        return Err(format!("frame mismatch: expected {frame:?}, received {received:?}").into());
    }
    if received.timestamp.unix_micros.is_some() {
        return Err("unexpected UTC timestamp on Gen2".into());
    }
    Ok(())
}
fn bidirectional(a: &mut Device, b: &mut Device, frame: &Frame) -> TestResult {
    transfer(a, b, frame)?;
    transfer(b, a, frame)
}
fn probe(a: &mut Device, b: &mut Device) -> TestResult {
    bidirectional(
        a,
        b,
        &Frame::classic(Id::standard(0x321)?, &[0x55, 0xaa, 0, 255])?,
    )
}
fn identifiers() -> TestResult<[Id; 4]> {
    Ok([
        Id::standard(0)?,
        Id::standard(0x7ff)?,
        Id::extended(0)?,
        Id::extended(0x1fff_ffff)?,
    ])
}
fn payload(size: usize) -> Vec<u8> {
    (0..size).map(|n| (n * 73 + size) as u8).collect()
}
fn filtered_one_way(
    a: &mut Device,
    b: &mut Device,
    filters: ReceiveFilters,
    accepted: &[Frame],
    rejected: &[Frame],
) -> TestResult {
    let cfg = ChannelConfig {
        fd: Some(FdMode::Iso),
        ..config()
    };
    configure(
        a,
        b,
        cfg,
        ChannelConfig {
            receive_filters: filters,
            ..cfg
        },
    )?;
    for frame in rejected {
        a.send(0, frame, TIMEOUT)?;
        if let Some(received) = b.receive(Duration::from_millis(30))? {
            return Err(format!("filter accepted excluded frame: {received:?}").into());
        }
    }
    for frame in accepted {
        transfer(a, b, frame)?;
    }
    // Changing back to accept-all must invalidate the old second filter too.
    b.configure_channel(0, cfg)?;
    drain(b)?;
    for frame in rejected {
        transfer(a, b, frame)?;
    }
    healthy(a)?;
    healthy(b)
}

fn filtered(
    a: &mut Device,
    b: &mut Device,
    filters: ReceiveFilters,
    accepted: &[Frame],
    rejected: &[Frame],
) -> TestResult {
    filtered_one_way(a, b, filters, accepted, rejected)?;
    filtered_one_way(b, a, filters, accepted, rejected)
}

fn main() -> TestResult {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 2 || args[0] == args[1] {
        return Err("provide two distinct USB serial numbers".into());
    }
    let devices: Vec<_> = busmust::devices()?.collect();
    let find = |serial: &str| -> TestResult<DeviceInfo> {
        let mut matches = devices.iter().filter(|d| d.serial_number() == Some(serial));
        let info = matches.next().ok_or("requested adapter not found")?.clone();
        if matches.next().is_some() {
            return Err("ambiguous USB serial".into());
        }
        if info.generation() != busmust::Generation::Gen2 || info.channel_count() != 1 {
            return Err("this sweep expects two Gen2 single-channel adapters".into());
        }
        Ok(info)
    };
    let a = find(&args[0])?;
    let b = find(&args[1])?;
    let report_path = if args.get(2).is_some_and(|selection| !selection.is_empty()) {
        "hardware-results-focused.tsv"
    } else {
        "hardware-results.tsv"
    };
    let mut report = File::create(report_path)?;
    writeln!(report, "# adapters\t{}\t{}", args[0], args[1])?;
    writeln!(report, "case\toutcome\telapsed_ms\tdetail")?;
    let mut sweep = Sweep {
        a,
        b,
        selected: args.get(2).cloned().unwrap_or_default(),
        report,
        passed: 0,
        failed: 0,
    };

    sweep.run("discovery-and-lifecycle", |a, b| {
        for device in [&mut *a, &mut *b] {
            if device.info().model() != "X1" || device.info().vendor_id() != 0x0810 {
                return Err("unexpected adapter metadata".into());
            }
            println!(
                "{} firmware {:?}",
                device.info().serial_number().unwrap_or("?"),
                device.firmware_version()
            );
            if device.channel_config(0)?.is_some() {
                return Err("fresh device has active configuration".into());
            }
            if !matches!(device.status(1), Err(Error::InvalidChannel(1))) {
                return Err("invalid channel accepted".into());
            }
            if !matches!(
                device.send(0, &Frame::classic(Id::standard(1)?, &[])?, TIMEOUT),
                Err(Error::ChannelInactive(0))
            ) {
                return Err("inactive send accepted".into());
            }
        }
        configure(a, b, config(), config())?;
        probe(a, b)?;
        for device in [&mut *a, &mut *b] {
            if device.receive(Duration::ZERO)?.is_some() {
                return Err("unexpected extra frame".into());
            }
            let start = Instant::now();
            if device.receive(Duration::from_millis(20))?.is_some()
                || start.elapsed() > Duration::from_millis(250)
            {
                return Err("receive timeout contract violated".into());
            }
            if device.recover_bus_off(0)?.bus_off {
                return Err("healthy recovery failed".into());
            }
            if device
                .configure_channel(
                    0,
                    ChannelConfig {
                        bitrate: 0,
                        ..config()
                    },
                )
                .is_ok()
            {
                return Err("invalid bitrate accepted".into());
            }
            if device.channel_config(0)? != Some(&config()) {
                return Err("invalid configuration changed state".into());
            }
            if device
                .send(0, &Frame::classic(Id::standard(1)?, &[])?, Duration::ZERO)
                .is_ok()
            {
                return Err("zero send timeout accepted".into());
            }
        }
        probe(a, b)?;
        a.stop_channel(0)?;
        a.stop_channel(0)?;
        if a.channel_config(0)?.is_some() {
            return Err("stopped channel still active".into());
        }
        a.configure_channel(0, config())?;
        probe(a, b)
    })?;

    for bitrate in [
        10_000, 20_000, 50_000, 100_000, 125_000, 250_000, 500_000, 800_000, 1_000_000,
    ] {
        sweep.run(&format!("classic-{bitrate}"), |a, b| {
            let cfg = ChannelConfig {
                bitrate,
                ..config()
            };
            configure(a, b, cfg, cfg)?;
            for id in identifiers()? {
                for size in 0..=8 {
                    bidirectional(a, b, &Frame::classic(id, &payload(size))?)?;
                }
                for size in 0..=8 {
                    bidirectional(a, b, &Frame::remote(id, size)?)?;
                }
            }
            healthy(a)?;
            healthy(b)
        })?;
    }

    for bitrate in [125_000, 250_000, 500_000, 1_000_000] {
        for data_bitrate in [
            500_000, 1_000_000, 2_000_000, 4_000_000, 5_000_000, 8_000_000,
        ] {
            if data_bitrate < bitrate {
                continue;
            }
            sweep.run(&format!("fd-iso-{bitrate}-{data_bitrate}"), |a, b| {
                let cfg = ChannelConfig {
                    bitrate,
                    data_bitrate,
                    fd: Some(FdMode::Iso),
                    ..config()
                };
                configure(a, b, cfg, cfg)?;
                for id in identifiers()? {
                    for bitrate_switch in [false, true] {
                        for size in [0, 1, 8, 12, 16, 20, 24, 32, 48, 64] {
                            bidirectional(
                                a,
                                b,
                                &Frame::fd(id, &payload(size), FdOptions { bitrate_switch })?,
                            )?;
                        }
                    }
                    bidirectional(a, b, &Frame::classic(id, &[1, 2, 3])?)?;
                }
                healthy(a)?;
                healthy(b)
            })?;
        }
    }

    sweep.run("fd-all-payload-lengths", |a, b| {
        let cfg = ChannelConfig {
            fd: Some(FdMode::Iso),
            ..config()
        };
        configure(a, b, cfg, cfg)?;
        for size in 0..=64 {
            for id in [Id::standard(0x456)?, Id::extended(0x18ff50e5)?] {
                bidirectional(
                    a,
                    b,
                    &Frame::fd(
                        id,
                        &payload(size),
                        FdOptions {
                            bitrate_switch: true,
                        },
                    )?,
                )?;
            }
        }
        healthy(a)?;
        healthy(b)
    })?;
    sweep.run("fd-non-iso", |a, b| {
        let cfg = ChannelConfig {
            fd: Some(FdMode::NonIso),
            ..config()
        };
        configure(a, b, cfg, cfg)?;
        for size in [0, 8, 12, 16, 20, 24, 32, 48, 64] {
            bidirectional(
                a,
                b,
                &Frame::fd(
                    Id::extended(0x18ff1234)?,
                    &payload(size),
                    FdOptions {
                        bitrate_switch: true,
                    },
                )?,
            )?;
        }
        healthy(a)?;
        healthy(b)
    })?;

    for (name, ta, tb) in [
        ("both-120", Termination::Ohms120, Termination::Ohms120),
        ("a-only", Termination::Ohms120, Termination::Disabled),
        ("b-only", Termination::Disabled, Termination::Ohms120),
        (
            "both-disabled",
            Termination::Disabled,
            Termination::Disabled,
        ),
    ] {
        for (protocol, bitrate, data_bitrate) in [
            ("classic", 125_000, 125_000),
            ("classic", 500_000, 500_000),
            ("classic", 1_000_000, 1_000_000),
            ("fd", 500_000, 2_000_000),
            ("fd", 1_000_000, 8_000_000),
        ] {
            sweep.run(
                &format!("termination-{name}-{protocol}-{bitrate}-{data_bitrate}"),
                |a, b| {
                    let cfg = ChannelConfig {
                        bitrate,
                        data_bitrate,
                        fd: (protocol == "fd").then_some(FdMode::Iso),
                        ..config()
                    };
                    configure(
                        a,
                        b,
                        ChannelConfig {
                            termination: ta,
                            ..cfg
                        },
                        ChannelConfig {
                            termination: tb,
                            ..cfg
                        },
                    )?;
                    let frame = if cfg.fd.is_some() {
                        Frame::fd(
                            Id::standard(0x321)?,
                            &[0x55; 64],
                            FdOptions {
                                bitrate_switch: true,
                            },
                        )?
                    } else {
                        Frame::classic(Id::standard(0x321)?, &[0x55; 8])?
                    };
                    for _ in 0..10 {
                        bidirectional(a, b, &frame)?;
                    }
                    configure(
                        a,
                        b,
                        ChannelConfig {
                            termination: Termination::Unchanged,
                            ..cfg
                        },
                        ChannelConfig {
                            termination: Termination::Unchanged,
                            ..cfg
                        },
                    )?;
                    bidirectional(a, b, &frame)?;
                    healthy(a)?;
                    healthy(b)
                },
            )?;
        }
    }

    for sample_point in [60, 75, 80, 87, 90] {
        sweep.run(&format!("sample-point-{sample_point}"), |a, b| {
            let cfg = ChannelConfig {
                fd: Some(FdMode::Iso),
                sample_point,
                data_sample_point: sample_point,
                ..config()
            };
            configure(a, b, cfg, cfg)?;
            bidirectional(
                a,
                b,
                &Frame::fd(
                    Id::standard(0x123)?,
                    &[0x55; 64],
                    FdOptions {
                        bitrate_switch: true,
                    },
                )?,
            )?;
            healthy(a)?;
            healthy(b)
        })?;
    }

    sweep.run("filters-standard-mask", |a, b| {
        let make = |id| Frame::classic(Id::standard(id).unwrap(), &[1, 2]).unwrap();
        filtered(
            a,
            b,
            ReceiveFilters::Match(Filter::new(Id::standard(0x120)?, 0x7f0)?),
            &[make(0x120), make(0x123), make(0x12f)],
            &[
                make(0x11f),
                make(0x130),
                Frame::classic(Id::extended(0x123)?, &[1, 2])?,
            ],
        )
    })?;
    sweep.run("filters-extended-exact", |a, b| {
        let id = Id::extended(0x18ff50e5)?;
        filtered(
            a,
            b,
            ReceiveFilters::Match(Filter::exact(id)),
            &[Frame::classic(id, &[3])?],
            &[
                Frame::classic(Id::extended(0x18ff50e4)?, &[3])?,
                Frame::classic(Id::standard(0x123)?, &[3])?,
            ],
        )
    })?;
    sweep.run("filters-extended-mask", |a, b| {
        let make = |id| Frame::classic(Id::extended(id).unwrap(), &[1, 2]).unwrap();
        filtered(
            a,
            b,
            ReceiveFilters::Match(Filter::new(Id::extended(0x18ff50a5)?, 0x1fffff00)?),
            &[make(0x18ff5000), make(0x18ff50e5), make(0x18ff50ff)],
            &[make(0x18ff5100), make(0x18ff4fff)],
        )
    })?;
    sweep.run("filters-two-slots", |a, b| {
        let s = Id::standard(0x123)?;
        let e = Id::extended(0x18ff50e5)?;
        filtered(
            a,
            b,
            ReceiveFilters::Either(Filter::exact(s), Filter::exact(e)),
            &[Frame::classic(s, &[1])?, Frame::classic(e, &[2])?],
            &[
                Frame::classic(Id::standard(0x124)?, &[1])?,
                Frame::classic(Id::extended(0x18ff50e4)?, &[2])?,
            ],
        )
    })?;
    sweep.run("filters-remote", |a, b| {
        let id = Id::standard(0x123)?;
        filtered(
            a,
            b,
            ReceiveFilters::Match(Filter::exact(id).with_remote(true)),
            &[Frame::remote(id, 8)?],
            &[Frame::classic(id, &[0; 8])?],
        )
    })?;
    sweep.run("filters-fd-brs", |a, b| {
        let id = Id::standard(0x123)?;
        filtered(
            a,
            b,
            ReceiveFilters::Match(Filter::exact(id).with_fd(true).with_bitrate_switch(true)),
            &[Frame::fd(
                id,
                &[0; 12],
                FdOptions {
                    bitrate_switch: true,
                },
            )?],
            &[
                Frame::classic(id, &[0; 8])?,
                Frame::fd(id, &[0; 12], FdOptions::default())?,
            ],
        )
    })?;

    sweep.run("echo-and-one-shot", |a, b| {
        let cfg = ChannelConfig {
            one_shot: true,
            receive_own_messages: true,
            ..config()
        };
        configure(a, b, cfg, cfg)?;
        for reverse in [false, true] {
            let (sender, receiver) = if reverse {
                (&mut *b, &mut *a)
            } else {
                (&mut *a, &mut *b)
            };
            let frame = Frame::classic(Id::standard(0x123)?, &[1, 2, 3])?;
            transfer(sender, receiver, &frame)?;
            let echo = sender.receive(TIMEOUT)?.ok_or("missing transmit echo")?;
            if !echo.is_echo || echo.frame != frame {
                return Err(format!("invalid transmit echo {echo:?}").into());
            }
        }
        healthy(a)?;
        healthy(b)
    })?;
    sweep.run("fd-echo-and-one-shot", |a, b| {
        let cfg = ChannelConfig {
            fd: Some(FdMode::Iso),
            one_shot: true,
            receive_own_messages: true,
            ..config()
        };
        configure(a, b, cfg, cfg)?;
        for reverse in [false, true] {
            let (sender, receiver) = if reverse {
                (&mut *b, &mut *a)
            } else {
                (&mut *a, &mut *b)
            };
            let frame = Frame::fd(
                Id::extended(0x18ff50e5)?,
                &[0xa5; 64],
                FdOptions {
                    bitrate_switch: true,
                },
            )?;
            transfer(sender, receiver, &frame)?;
            let echo = sender
                .receive(TIMEOUT)?
                .ok_or("missing CAN FD transmit echo")?;
            if !echo.is_echo || echo.frame != frame {
                return Err("CAN FD echo mismatch".into());
            }
        }
        healthy(a)?;
        healthy(b)
    })?;
    sweep.run("fd-esi-active-state", |a, b| {
        let cfg = ChannelConfig {
            fd: Some(FdMode::Iso),
            ..config()
        };
        configure(a, b, cfg, cfg)?;
        let frame = Frame::fd(
            Id::standard(0x123)?,
            &[0x55; 12],
            FdOptions {
                bitrate_switch: true,
            },
        )?;
        bidirectional(a, b, &frame)?;
        healthy(a)?;
        healthy(b)
    })?;
    sweep.run("listen-only-and-reconfigure", |a, b| {
        configure(
            a,
            b,
            config(),
            ChannelConfig {
                mode: Mode::ListenOnly,
                ..config()
            },
        )?;
        if !matches!(
            b.send(0, &Frame::classic(Id::standard(1)?, &[])?, TIMEOUT),
            Err(Error::ListenOnly)
        ) {
            return Err("listen-only send was not rejected".into());
        }
        b.configure_channel(0, config())?;
        probe(a, b)?;
        healthy(a)?;
        healthy(b)
    })?;
    sweep.run("burst-order-and-timestamps", |a, b| {
        configure(a, b, config(), config())?;
        for sequence in 0u32..256 {
            a.send(
                0,
                &Frame::classic(Id::standard(0x321)?, &sequence.to_le_bytes())?,
                TIMEOUT,
            )?;
        }
        let mut last = None;
        for sequence in 0u32..256 {
            let frame = b.receive(TIMEOUT)?.ok_or("burst frame missing")?;
            if frame.frame.data() != sequence.to_le_bytes() {
                return Err(format!("burst order mismatch at {sequence}").into());
            }
            let ticks = frame.timestamp.device_micros;
            if last.is_some_and(|previous: u32| ticks.wrapping_sub(previous) > 0x7fff_ffff) {
                return Err("device timestamps moved backwards".into());
            }
            last = Some(ticks);
        }
        if b.receive(Duration::from_millis(10))?.is_some() {
            return Err("unexpected duplicate burst frame".into());
        }
        healthy(a)?;
        healthy(b)
    })?;

    sweep.run("internal-loopback-isolation", |a, b| {
        let cfg = ChannelConfig {
            mode: Mode::InternalLoopback,
            fd: Some(FdMode::Iso),
            ..config()
        };
        configure(a, b, cfg, cfg)?;
        for reverse in [false, true] {
            let (sender, peer) = if reverse {
                (&mut *b, &mut *a)
            } else {
                (&mut *a, &mut *b)
            };
            for size in [0, 8, 12, 64] {
                let frame = Frame::fd(
                    Id::extended(0x18ff50e5)?,
                    &payload(size),
                    FdOptions {
                        bitrate_switch: true,
                    },
                )?;
                sender.send(0, &frame, TIMEOUT)?;
                let received = sender
                    .receive(TIMEOUT)?
                    .ok_or("missing internal loopback")?;
                if received.frame != frame {
                    return Err("internal loopback payload mismatch".into());
                }
            }
            if peer.receive(Duration::from_millis(20))?.is_some() {
                return Err("internal loopback leaked onto physical bus".into());
            }
        }
        Ok(())
    })?;
    sweep.run("induced-bus-off-and-legacy-recovery", |a, b| {
        let cfg = ChannelConfig {
            fd: Some(FdMode::Iso),
            ..config()
        };
        configure(
            a,
            b,
            cfg,
            ChannelConfig {
                data_bitrate: 4_000_000,
                ..cfg
            },
        )?;
        let frame = Frame::fd(
            Id::standard(0x123)?,
            &[0x55; 64],
            FdOptions {
                bitrate_switch: true,
            },
        )?;
        a.send(0, &frame, TIMEOUT)?;
        let start = Instant::now();
        while !a.status(0)?.bus_off {
            if start.elapsed() > Duration::from_secs(3) {
                return Err("mismatched data rates did not induce transmitter bus-off".into());
            }
            match a.send(0, &frame, TIMEOUT) {
                Ok(()) => {}
                Err(Error::BusOff(0)) => break,
                Err(error) => return Err(error.into()),
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        if !matches!(a.send(0, &frame, TIMEOUT), Err(Error::BusOff(0))) {
            return Err("bus-off send was not rejected".into());
        }
        b.stop_channel(0)?;
        if a.recover_bus_off(0)?.bus_off {
            return Err("legacy recovery left controller bus-off".into());
        }
        if a.channel_config(0)? != Some(&cfg) {
            return Err("recovery changed requested configuration".into());
        }
        b.configure_channel(0, cfg)?;
        drain(a)?;
        drain(b)?;
        bidirectional(a, b, &frame)?;
        healthy(a)?;
        healthy(b)
    })?;

    let selected_cases = sweep.passed + sweep.failed;
    // Always leave both adapters stopped with their termination enabled.
    sweep.run("final-reopen-and-cleanup", |a, b| {
        configure(a, b, config(), config())?;
        probe(a, b)?;
        healthy(a)?;
        healthy(b)
    })?;
    println!(
        "TOTAL {} passed, {} failed; report: {report_path}",
        sweep.passed, sweep.failed
    );
    if selected_cases == 0 {
        return Err("no cases matched selection".into());
    }
    if sweep.failed != 0 {
        return Err(format!("{} hardware cases failed", sweep.failed).into());
    }
    Ok(())
}
