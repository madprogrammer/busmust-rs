use super::*;
use crate::{FdMode, FdOptions};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq)]
enum Operation {
    Read(u8, u8, u16),
    Control(u8, u16, u8, Vec<u8>),
    Write(Vec<u8>),
    Receive,
    Settle,
    Delay(Duration),
    FlushRead,
}
#[derive(Default)]
struct Fake {
    operations: Rc<RefCell<Vec<Operation>>>,
    responses: VecDeque<Result<Vec<u8>>>,
    reads: VecDeque<Result<Option<Vec<u8>>>>,
    fail_control_at: Option<usize>,
    control_count: usize,
    fail_write: bool,
    partial_reads: VecDeque<Result<Vec<u8>>>,
}
impl Transport for Fake {
    fn control_in(&mut self, request: u8, channel: u8, length: u16) -> Result<Vec<u8>> {
        self.operations
            .borrow_mut()
            .push(Operation::Read(request, channel, length));
        self.responses
            .pop_front()
            .unwrap_or_else(|| Ok(vec![0; usize::from(length)]))
    }
    fn control_out(&mut self, request: u8, value: u16, channel: u8, data: &[u8]) -> Result<()> {
        self.operations.borrow_mut().push(Operation::Control(
            request,
            value,
            channel,
            data.to_vec(),
        ));
        let index = self.control_count;
        self.control_count += 1;
        if self.fail_control_at == Some(index) {
            return Err(Error::Transfer(nusb::transfer::TransferError::Disconnected));
        }
        Ok(())
    }
    fn write(&mut self, data: &[u8], _: Duration) -> Result<()> {
        self.operations
            .borrow_mut()
            .push(Operation::Write(data.to_vec()));
        if self.fail_write {
            return Err(Error::ShortTransfer {
                expected: data.len(),
                actual: 3,
            });
        }
        Ok(())
    }
    fn read(&mut self, _: Duration) -> Result<Option<Vec<u8>>> {
        self.operations.borrow_mut().push(Operation::Receive);
        self.reads.pop_front().unwrap_or(Ok(None))
    }
    fn flush_read(&mut self) -> Result<Vec<u8>> {
        self.operations.borrow_mut().push(Operation::FlushRead);
        self.partial_reads.pop_front().unwrap_or(Ok(Vec::new()))
    }
    fn delay(&mut self, duration: Duration) {
        self.operations
            .borrow_mut()
            .push(Operation::Delay(duration));
    }
    fn settle(&mut self) {
        self.operations.borrow_mut().push(Operation::Settle);
    }
}
fn connection() -> Connection<Fake> {
    Connection::new(Fake::default(), product(0xf023).unwrap())
}
fn frame() -> Frame {
    Frame::classic(crate::Id::standard(0x123).unwrap(), &[1, 2]).unwrap()
}
fn bus_off() -> Vec<u8> {
    vec![1, 0, 0, 0, 0, 0, 255, 0]
}

#[test]
fn configuration_order_and_wire_values() {
    let mut c = connection();
    c.configure(
        1,
        ChannelConfig {
            fd: Some(FdMode::Iso),
            termination: Termination::Ohms120,
            ..Default::default()
        },
    )
    .unwrap();
    let mut filter = vec![0; 32];
    filter[0] = 1;
    assert_eq!(
        *c.transport.operations.borrow(),
        [
            Operation::Control(0xc0, 4, 1, vec![]),
            Operation::Control(0xc2, 0, 1, vec![0xf4, 1, 0xd0, 7, 87, 80, 0, 0, 0, 0, 0, 0]),
            Operation::Settle,
            Operation::Control(0xc8, 0, 1, filter),
            Operation::Control(0xc8, 1, 1, vec![0; 32]),
            Operation::Control(0xc3, 120, 1, vec![]),
            Operation::Control(0xc0, 0, 1, vec![]),
            Operation::Settle,
            Operation::Read(0xd1, 1, 8),
        ]
    );
}

#[test]
fn invalid_configuration_does_not_touch_usb_or_existing_state() {
    let mut c = connection();
    c.configure(0, ChannelConfig::default()).unwrap();
    c.transport.operations.borrow_mut().clear();
    assert!(matches!(
        c.configure(2, ChannelConfig::default()),
        Err(Error::InvalidChannel(2))
    ));
    assert!(c
        .configure(
            0,
            ChannelConfig {
                bitrate: 0,
                ..Default::default()
            }
        )
        .is_err());
    assert!(c.transport.operations.borrow().is_empty());
    assert!(c.channels[0].is_some());
}

#[test]
fn every_configuration_failure_stops_channel() {
    for index in 0..6 {
        let mut c = connection();
        c.transport.fail_control_at = Some(index);
        assert!(c
            .configure(
                0,
                ChannelConfig {
                    termination: Termination::Disabled,
                    ..Default::default()
                }
            )
            .is_err());
        assert!(c.channels[0].is_none());
        assert_eq!(
            c.transport.operations.borrow().last(),
            Some(&Operation::Control(0xc0, 4, 0, vec![]))
        );
        assert!(!c.touched[0]);
    }
}

#[test]
fn close_and_drop_stop_all_channels_even_after_error() {
    let mut c = connection();
    c.configure(0, ChannelConfig::default()).unwrap();
    c.configure(1, ChannelConfig::default()).unwrap();
    c.transport.operations.borrow_mut().clear();
    c.transport.fail_control_at = Some(c.transport.control_count);
    assert!(c.shutdown().is_err());
    assert_eq!(
        *c.transport.operations.borrow(),
        [
            Operation::Control(0xc0, 4, 0, vec![]),
            Operation::Control(0xc0, 4, 1, vec![]),
        ]
    );
    let operations = Rc::clone(&c.transport.operations);
    drop(c);
    assert_eq!(operations.borrow().len(), 3); // Retry only the unsuccessful stop.
}

#[test]
fn normal_drop_releases_active_channels_and_stop_is_idempotent() {
    let mut c = connection();
    c.configure(0, ChannelConfig::default()).unwrap();
    c.configure(1, ChannelConfig::default()).unwrap();
    c.stop(0).unwrap();
    c.transport.operations.borrow_mut().clear();
    c.stop(0).unwrap();
    let operations = Rc::clone(&c.transport.operations);
    drop(c);
    assert_eq!(
        *operations.borrow(),
        [Operation::Control(0xc0, 4, 1, vec![])]
    );
}

#[test]
fn send_checks_channel_mode_fd_and_timeout_before_usb() {
    let mut c = connection();
    assert!(matches!(
        c.send(0, &frame(), Duration::from_secs(1)),
        Err(Error::ChannelInactive(0))
    ));
    c.configure(
        0,
        ChannelConfig {
            mode: Mode::ListenOnly,
            ..Default::default()
        },
    )
    .unwrap();
    c.transport.operations.borrow_mut().clear();
    assert!(matches!(
        c.send(0, &frame(), Duration::from_secs(1)),
        Err(Error::ListenOnly)
    ));
    assert!(c.transport.operations.borrow().is_empty());
    c.configure(0, ChannelConfig::default()).unwrap();
    c.transport.operations.borrow_mut().clear();
    assert!(c.send(0, &frame(), Duration::ZERO).is_err());
    let fd = Frame::fd(frame().id(), &[], FdOptions::default()).unwrap();
    assert!(c.send(0, &fd, Duration::from_secs(1)).is_err());
    assert!(c.transport.operations.borrow().is_empty());
}

#[test]
fn successful_send_requests_echo_and_checks_bus_off() {
    let mut c = connection();
    c.configure(
        0,
        ChannelConfig {
            receive_own_messages: true,
            ..Default::default()
        },
    )
    .unwrap();
    c.transport.operations.borrow_mut().clear();
    c.send(0, &frame(), Duration::from_secs(1)).unwrap();
    assert_eq!(
        *c.transport.operations.borrow(),
        [
            Operation::Read(0xd1, 0, 8),
            Operation::Write(protocol::encode(0, &frame(), true))
        ]
    );
    c.transport.responses.push_back(Ok(bus_off()));
    assert!(matches!(
        c.send(0, &frame(), Duration::from_secs(1)),
        Err(Error::BusOff(0))
    ));
}

#[test]
fn failed_write_is_not_retried_and_poisoned_stream_rejects_further_sends() {
    let mut c = connection();
    c.configure(0, ChannelConfig::default()).unwrap();
    c.transport.fail_write = true;
    assert!(matches!(
        c.send(0, &frame(), Duration::from_secs(1)),
        Err(Error::ShortTransfer { .. })
    ));
    c.transport.operations.borrow_mut().clear();
    assert!(matches!(
        c.send(0, &frame(), Duration::from_secs(1)),
        Err(Error::TransmitFailed)
    ));
    assert!(c.transport.operations.borrow().is_empty());
}

#[test]
fn fragmented_read_survives_timeout_and_keeps_sibling_frames() {
    let mut c = connection();
    c.configure(0, ChannelConfig::default()).unwrap();
    c.configure(1, ChannelConfig::default()).unwrap();
    let first = protocol::encode(1, &frame(), false);
    c.transport.reads.push_back(Ok(Some(first[..5].to_vec())));
    c.transport.reads.push_back(Ok(None));
    assert!(c.receive(Duration::from_secs(1)).unwrap().is_none());
    let mut rest = first[5..].to_vec();
    rest.extend(protocol::encode(0, &frame(), false));
    c.transport.reads.push_back(Ok(Some(rest)));
    assert_eq!(
        c.receive(Duration::from_secs(1)).unwrap().unwrap().channel,
        1
    );
    c.transport.operations.borrow_mut().clear();
    assert_eq!(c.receive(Duration::ZERO).unwrap().unwrap().channel, 0);
    assert!(c.transport.operations.borrow().is_empty());
}

#[test]
fn ignores_inactive_channels_and_unrequested_echoes() {
    let mut c = connection();
    c.configure(0, ChannelConfig::default()).unwrap();
    let mut echo = protocol::encode(0, &frame(), false);
    echo[0] = 10;
    let mut bytes = protocol::encode(1, &frame(), false);
    bytes.extend(&echo);
    bytes.extend(protocol::encode(0, &frame(), false));
    c.transport.reads.push_back(Ok(Some(bytes)));
    let received = c.receive(Duration::ZERO).unwrap().unwrap();
    assert_eq!(received.channel, 0);
    assert!(!received.is_echo);
    assert!(c.receive(Duration::ZERO).unwrap().is_none());
    c.configure(
        0,
        ChannelConfig {
            receive_own_messages: true,
            ..Default::default()
        },
    )
    .unwrap();
    c.transport.reads.push_back(Ok(Some(echo)));
    assert!(c.receive(Duration::ZERO).unwrap().unwrap().is_echo);
}

#[test]
fn receive_errors_poison_the_stream() {
    for error in [false, true] {
        let mut c = connection();
        c.transport.reads.push_back(if error {
            Err(Error::Transfer(nusb::transfer::TransferError::Disconnected))
        } else {
            Ok(Some(vec![2, 0, 1, 4, 0, 0, 0, 0]))
        });
        assert!(c.receive(Duration::ZERO).is_err());
        c.transport.operations.borrow_mut().clear();
        assert!(matches!(
            c.receive(Duration::ZERO),
            Err(Error::ReceiveFailed)
        ));
        assert!(c.transport.operations.borrow().is_empty());
    }
}

#[test]
fn status_checks_length_and_decodes_error_counters() {
    let mut c = connection();
    c.transport
        .responses
        .push_back(Ok(vec![1, 99, 1, 0, 0, 1, 123, 45]));
    let status = c.status(0).unwrap();
    assert_eq!(
        status,
        CanStatus {
            bus_off: true,
            tx_passive: true,
            rx_passive: false,
            tx_warning: false,
            rx_warning: true,
            tx_error_counter: 123,
            rx_error_counter: 45
        }
    );
    c.transport.responses.push_back(Ok(vec![0; 7]));
    assert!(matches!(
        c.status(0),
        Err(Error::ShortTransfer {
            expected: 8,
            actual: 7
        })
    ));
}

#[test]
fn product_table_matches_reference_channel_counts() {
    for (pid, count) in [
        (0xf012, 1),
        (0xf112, 1),
        (0xf122, 2),
        (0xf142, 4),
        (0xf182, 8),
        (0xe122, 2),
        (0xe142, 4),
        (0xf013, 1),
        (0xf023, 2),
        (0xf043, 4),
        (0x0043, 2),
        (0x0083, 4),
    ] {
        assert_eq!(product(pid).unwrap().channels, count);
    }
    assert!(product(0xffff).is_none());
}

#[test]
fn controller_mode_and_classic_bitrate_encoding() {
    let mut config = ChannelConfig::default();
    assert_eq!(config.wire_mode(), 6);
    assert_eq!(&config.bitrate_payload()[..4], &[0xf4, 1, 0xf4, 1]);
    config.fd = Some(FdMode::NonIso);
    config.one_shot = true;
    assert_eq!(config.wire_mode(), 24);
    config.mode = Mode::ListenOnly;
    assert_eq!(config.wire_mode(), 27);
    config.mode = Mode::InternalLoopback;
    assert_eq!(config.wire_mode(), 26);
}

fn configured_gen2() -> Connection<Fake> {
    let mut c = Connection::new(Fake::default(), product(0xf012).unwrap());
    c.configure(
        0,
        ChannelConfig {
            bitrate: 250_000,
            mode: Mode::InternalLoopback,
            ..Default::default()
        },
    )
    .unwrap();
    c.transport.operations.borrow_mut().clear();
    c
}

#[test]
fn legacy_gen2_recovery_matches_sequence_and_restores_settings() {
    let mut c = configured_gen2();
    c.firmware = Some([2, 5, 0, 0]);
    c.transport.responses.push_back(Ok(bus_off()));
    assert!(!c.recover(0).unwrap().bus_off);
    let config = c.config(0).unwrap();
    let operations = c.transport.operations.borrow();
    let dummy = protocol::encode(
        0,
        &Frame::remote(crate::Id::standard(0).unwrap(), 0).unwrap(),
        false,
    )
    .repeat(256);
    assert_eq!(dummy.len(), 4096);
    assert_eq!(
        *operations,
        [
            Operation::Read(0xd1, 0, 8),
            Operation::Control(0xc0, 4, 0, vec![]),
            Operation::Control(
                0xc2,
                0,
                0,
                vec![0xe8, 3, 0x40, 0x1f, 87, 75, 0, 0, 0, 0, 0, 0]
            ),
            Operation::Settle,
            Operation::Control(0xc0, 2, 0, vec![]),
            Operation::Settle,
            Operation::Write(dummy),
            Operation::Delay(Duration::from_millis(50)),
            Operation::Control(0xc0, 4, 0, vec![]),
            Operation::Settle,
            Operation::Receive,
            Operation::FlushRead,
            Operation::Control(0xc0, 4, 0, vec![]),
            Operation::Control(0xc2, 0, 0, config.bitrate_payload().to_vec()),
            Operation::Settle,
            Operation::Control(0xc0, config.wire_mode(), 0, vec![]),
            Operation::Settle,
            Operation::Read(0xd1, 0, 8),
        ]
    );
}

#[test]
fn startup_recovers_gen2_with_unknown_firmware() {
    let mut c = Connection::new(Fake::default(), product(0xf012).unwrap());
    c.transport.responses.push_back(Ok(bus_off()));
    c.configure(0, ChannelConfig::default()).unwrap();
    assert!(c.channels[0].is_some());
    assert!(c
        .transport
        .operations
        .borrow()
        .iter()
        .any(|op| matches!(op, Operation::Write(bytes) if bytes.len() == 4096)));
}

#[test]
fn recovery_uses_firmware_when_supported_and_falls_back_when_needed() {
    for (pid, version) in [
        (0xf012, [2, 6, 0, 0]),
        (0xe122, [2, 6, 0, 0]),
        (0xf023, [3, 1, 0, 0]),
    ] {
        for fallback in 0..3 {
            let mut c = Connection::new(Fake::default(), product(pid).unwrap());
            c.configure(0, ChannelConfig::default()).unwrap();
            c.firmware = Some(version);
            c.transport.operations.borrow_mut().clear();
            c.transport.responses.push_back(Ok(bus_off()));
            if fallback == 2 {
                c.transport
                    .responses
                    .push_back(Err(Error::Transfer(nusb::transfer::TransferError::Stall)));
            } else {
                c.transport.responses.push_back(Ok(vec![]));
                c.transport.responses.push_back(Ok(if fallback == 1 {
                    bus_off()
                } else {
                    vec![0; 8]
                }));
            }
            assert!(!c.recover(0).unwrap().bus_off);
            let operations = c.transport.operations.borrow();
            assert!(operations.contains(&Operation::Read(0xf5, 0, 0)));
            assert_eq!(
                operations
                    .iter()
                    .any(|op| matches!(op, Operation::Write(_))),
                fallback != 0
            );
        }
    }
}

#[test]
fn recovery_is_noop_when_healthy_and_bounded_when_bus_off_persists() {
    let mut c = configured_gen2();
    assert!(!c.recover(0).unwrap().bus_off);
    assert_eq!(c.transport.operations.borrow().len(), 1);
    c.transport.operations.borrow_mut().clear();
    for _ in 0..9 {
        c.transport.responses.push_back(Ok(bus_off()));
    }
    assert!(matches!(c.recover(0), Err(Error::BusOff(0))));
    assert!(c.channels[0].is_none());
    assert_eq!(
        c.transport
            .operations
            .borrow()
            .iter()
            .filter(|op| matches!(op, Operation::Write(_)))
            .count(),
        8
    );
}

#[test]
fn failed_recovery_write_restores_timing_and_deactivates_channel() {
    let mut c = configured_gen2();
    let timing = c.config(0).unwrap().bitrate_payload().to_vec();
    c.transport.responses.push_back(Ok(bus_off()));
    c.transport.fail_write = true;
    assert!(matches!(c.recover(0), Err(Error::ShortTransfer { .. })));
    assert!(c.channels[0].is_none());
    assert!(c.transmit_failed);
    let operations = c.transport.operations.borrow();
    assert!(operations.contains(&Operation::Control(0xc2, 0, 0, timing)));
    assert_eq!(
        operations.last(),
        Some(&Operation::Control(0xc0, 4, 0, vec![]))
    );
    assert_eq!(
        operations
            .iter()
            .filter(|op| matches!(op, Operation::Write(_)))
            .count(),
        1
    );
}

#[test]
fn recovery_control_failures_never_leave_the_channel_active() {
    for failure in 0..7 {
        let mut c = configured_gen2();
        c.transport.responses.push_back(Ok(bus_off()));
        c.transport.fail_control_at = Some(c.transport.control_count + failure);
        assert!(c.recover(0).is_err(), "failure index {failure}");
        assert!(c.channels[0].is_none());
        assert_eq!(
            c.transport.operations.borrow().last(),
            Some(&Operation::Control(0xc0, 4, 0, vec![]))
        );
    }
}

#[test]
fn recovery_preserves_sibling_frames_and_discards_fragmented_dummies() {
    let mut c = connection();
    c.configure(0, ChannelConfig::default()).unwrap();
    c.configure(1, ChannelConfig::default()).unwrap();
    c.transport.responses.push_back(Ok(bus_off()));
    let dummy = protocol::encode(
        0,
        &Frame::remote(crate::Id::standard(0).unwrap(), 0).unwrap(),
        false,
    );
    let sibling = protocol::encode(1, &frame(), false);
    let mut first = sibling.clone();
    first.extend(&dummy[..5]);
    c.transport.reads.push_back(Ok(Some(first)));
    // An incomplete USB transfer still contains the dummy remainder and a
    // sibling frame: flush_read must deliver both for decoding.
    c.transport.reads.push_back(Ok(None));
    let mut partial = dummy[5..].to_vec();
    partial.extend(&sibling);
    c.transport.partial_reads.push_back(Ok(partial));
    assert!(!c.recover(0).unwrap().bus_off);
    assert_eq!(c.receive(Duration::ZERO).unwrap().unwrap().channel, 1);
    assert_eq!(c.receive(Duration::ZERO).unwrap().unwrap().channel, 1);
    assert!(c.receive(Duration::ZERO).unwrap().is_none());
}

#[test]
fn recovery_receive_failure_restores_timing_and_requires_reopen() {
    let mut c = configured_gen2();
    let timing = c.config(0).unwrap().bitrate_payload().to_vec();
    c.transport.responses.push_back(Ok(bus_off()));
    c.transport.reads.push_back(Err(Error::Transfer(
        nusb::transfer::TransferError::Disconnected,
    )));
    assert!(c.recover(0).is_err());
    assert!(c.receive_failed);
    assert!(c.channels[0].is_none());
    assert!(c
        .transport
        .operations
        .borrow()
        .contains(&Operation::Control(0xc2, 0, 0, timing)));
}

#[test]
fn recovery_drain_has_a_bound_even_with_continuous_usb_traffic() {
    let mut c = configured_gen2();
    c.transport.responses.push_back(Ok(bus_off()));
    for _ in 0..64 {
        c.transport
            .reads
            .push_back(Ok(Some(protocol::encode(0, &frame(), false))));
    }
    assert!(matches!(c.recover(0), Err(Error::RecoveryFailed(_))));
    assert!(c.receive_failed);
    assert!(c.channels[0].is_none());
}

#[test]
fn standard_and_extended_hardware_filter_encoding() {
    use crate::{Filter, Id, ReceiveFilters};
    let standard = Filter::new(Id::standard(0x123).unwrap(), 0x7f0)
        .unwrap()
        .with_remote(true);
    let extended = Filter::exact(Id::extended(0x18ff50e5).unwrap())
        .with_fd(true)
        .with_bitrate_switch(true);
    let mut c = connection();
    c.configure(
        0,
        ChannelConfig {
            receive_filters: ReceiveFilters::Either(standard, extended),
            ..Default::default()
        },
    )
    .unwrap();
    let mut a = vec![0; 32];
    a[0] = 1;
    a[2] = 3;
    a[3] = 2;
    a[8..12].copy_from_slice(&0x7f0u32.to_le_bytes());
    a[12..16].copy_from_slice(&0x120u32.to_le_bytes());
    let mut b = vec![0; 32];
    b[0] = 1;
    b[2] = 13;
    b[3] = 13;
    b[8..12].copy_from_slice(&0x1fff_ffffu32.to_le_bytes());
    b[12..16].copy_from_slice(&0x1a872e3fu32.to_le_bytes());
    assert!(c
        .transport
        .operations
        .borrow()
        .contains(&Operation::Control(0xc8, 0, 0, a)));
    assert!(c
        .transport
        .operations
        .borrow()
        .contains(&Operation::Control(0xc8, 1, 0, b)));
}

#[test]
fn receive_applies_flag_filters_that_gen2_hardware_ignores() {
    let mut c = connection();
    let id = frame().id();
    c.configure(
        0,
        ChannelConfig {
            receive_filters: crate::ReceiveFilters::Match(
                crate::Filter::exact(id).with_remote(true),
            ),
            ..Default::default()
        },
    )
    .unwrap();
    let mut bytes = protocol::encode(0, &frame(), false);
    let remote = Frame::remote(id, 8).unwrap();
    bytes.extend(protocol::encode(0, &remote, false));
    c.transport.reads.push_back(Ok(Some(bytes)));
    assert_eq!(c.receive(Duration::ZERO).unwrap().unwrap().frame, remote);
    assert!(c.receive(Duration::ZERO).unwrap().is_none());
}
