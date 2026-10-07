use super::*;

fn standard() -> Id {
    Id::standard(0x123).unwrap()
}
fn hex(input: &str) -> Vec<u8> {
    input
        .as_bytes()
        .chunks_exact(2)
        .map(|s| u8::from_str_radix(std::str::from_utf8(s).unwrap(), 16).unwrap())
        .collect()
}
fn envelope(header: u16, payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(header.to_le_bytes());
    bytes.extend((payload.len() as u16).to_le_bytes());
    bytes.extend(0xffff_fff0u32.to_le_bytes());
    bytes.extend(payload);
    bytes.resize(align4(bytes.len()), 0);
    bytes
}

#[test]
fn standard_wire_vector() {
    let frame = Frame::classic(standard(), &[0xde, 0xad]).unwrap();
    assert_eq!(
        encode(1, &frame, false),
        hex("02f10c00000000002301000002000000dead0000")
    );
}

#[test]
fn extended_fd_wire_vector_and_decode() {
    let data: Vec<_> = (0..12).collect();
    let frame = Frame::fd(
        Id::extended(0x18ff50e5).unwrap(),
        &data,
        FdOptions {
            bitrate_switch: true,
        },
    )
    .unwrap();
    let mut expected = hex("02f01400000000003f2e871ad9000000");
    expected.extend(data);
    assert_eq!(encode(0, &frame, false), expected);
    assert_eq!(Decoder::default().feed(&expected).unwrap()[0].frame, frame);
    expected[13] |= 1; // Incoming ESI describes the physical transmitter state.
    let received = Decoder::default().feed(&expected).unwrap().remove(0).frame;
    assert!(received.is_error_passive());
    assert_eq!(received.id(), frame.id());
    assert_eq!(received.data(), frame.data());
    assert_eq!(received.fd_options(), frame.fd_options());
    // Forwarding a received frame must not try to force the old transmitter's ESI.
    assert_eq!(encode(0, &received, false)[13], 0);
}

#[test]
fn every_fd_length_round_trips_with_zero_padding() {
    for size in 0..=64 {
        let data = vec![0xa5; size];
        let frame = Frame::fd(standard(), &data, FdOptions::default()).unwrap();
        let wire_size = usize::from(*LENGTHS.iter().find(|&&n| usize::from(n) >= size).unwrap());
        assert_eq!(frame.len(), wire_size);
        assert_eq!(&frame.data()[..size], data);
        assert!(frame.data()[size..].iter().all(|&b| b == 0));
        let encoded = encode(0, &frame, false);
        assert_eq!(encoded.len(), 16 + align4(wire_size));
        assert_eq!(Decoder::default().feed(&encoded).unwrap()[0].frame, frame);
    }
}

#[test]
fn remote_receive_has_no_data_but_preserves_requested_length() {
    let frame = Frame::remote(standard(), 8).unwrap();
    let tx = encode(0, &frame, false);
    assert_eq!(tx.len(), 24);
    assert_eq!(tx[12], 0x28);
    assert_eq!(&tx[16..], &[0; 8]);
    let received = Decoder::default()
        .feed(&envelope(0x0f02, &tx[8..16]))
        .unwrap()
        .remove(0);
    assert_eq!(received.frame, frame);
    assert_eq!(received.frame.len(), 8);
    assert!(received.frame.data().is_empty());
}

#[test]
fn every_fragment_boundary_and_concatenated_packets() {
    let first = encode(1, &Frame::classic(standard(), b"ab").unwrap(), false);
    let second = encode(0, &Frame::classic(standard(), b"cde").unwrap(), false);
    for split in 0..=first.len() {
        let mut decoder = Decoder::default();
        let mut frames = decoder.feed(&first[..split]).unwrap();
        let mut rest = first[split..].to_vec();
        rest.extend(&second);
        frames.extend(decoder.feed(&rest).unwrap());
        assert_eq!(frames.iter().map(|f| f.channel).collect::<Vec<_>>(), [1, 0]);
        assert_eq!(frames[1].frame.data(), b"cde");
        assert!(decoder.buffer.is_empty());
    }
    let mut decoder = Decoder::default();
    let mut frames = Vec::new();
    for byte in first {
        frames.extend(decoder.feed(&[byte]).unwrap());
    }
    assert_eq!(frames.len(), 1);
}

#[test]
fn routing_system_packets_group_packets_echo_and_timestamps() {
    let tx = encode(0, &Frame::classic(standard(), b"abc").unwrap(), true);
    assert_eq!(u32_at(&tx, 12) & ECHO, ECHO);
    let mut payload = tx[8..].to_vec();
    payload.extend([0; 8]);
    payload.extend(1_700_000_000_123_456u64.to_le_bytes());
    let mut stream = envelope(0xf, &[0; 4]); // SYSTEM contains CAN's bit.
    stream.extend(envelope(8, &[]));
    stream.extend(envelope(0x2022, &tx[8..])); // Group packet.
    stream.extend(envelope(0x2f1a, &payload)); // Echo, source channel 2, tail.
    stream.extend(envelope(0xf102, &tx[8..])); // Directed to channel 1.
    let frames = Decoder::default().feed(&stream).unwrap();
    assert_eq!(frames.len(), 2);
    assert!(frames[0].is_echo);
    assert_eq!(frames[0].channel, 2);
    assert_eq!(frames[0].timestamp.device_micros, 0xffff_fff0);
    assert_eq!(frames[0].timestamp.unix_micros, Some(1_700_000_000_123_456));
    assert_eq!(frames[1].channel, 1);
}

#[test]
fn missing_fdf_compatibility_and_zero_utc_tail() {
    let mut payload = vec![0; 36];
    payload[4] = 9;
    let frame = Decoder::default()
        .feed(&envelope(0x12, &payload))
        .unwrap()
        .remove(0);
    assert!(frame.frame.is_fd());
    assert_eq!(frame.frame.len(), 12);
    assert_eq!(frame.timestamp.unix_micros, None);
}

#[test]
fn malformed_packets_fail_without_panicking() {
    let bad = [
        hex("0200010400000000"),                   // Envelope length > 1024.
        envelope(2, &[0; 4]),                      // Missing control.
        envelope(0x12, &[0; 8]),                   // Missing tail.
        envelope(2, &[0, 0, 0, 0, 8, 0, 0, 0]),    // Missing data.
        envelope(2, &[0, 0, 0, 0, 0xa0, 0, 0, 0]), // RTR + FDF.
        envelope(2, &[0, 0, 0, 0, 0x29, 0, 0, 0]), // RTR DLC > 8.
        envelope(2, &[0, 0, 0, 0, 0x40, 0, 0, 0]), // Classic BRS.
    ];
    for bytes in bad {
        let mut decoder = Decoder::default();
        assert!(matches!(decoder.feed(&bytes), Err(Error::Protocol(_))));
        assert!(decoder.buffer.is_empty());
    }
}

#[test]
fn arbitrary_envelopes_do_not_panic() {
    // Deterministic malformed-input coverage without an external fuzz runtime.
    let mut state = 0x12345678u32;
    for length in 0..=1040 {
        let mut bytes = vec![0; length];
        for byte in &mut bytes {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            *byte = state as u8;
        }
        let _ = Decoder::default().feed(&bytes);
    }
}

#[test]
fn partial_recovery_envelope_is_discarded_without_losing_the_next_frame() {
    for channel in [0, 1] {
        let packet = encode(
            channel,
            &Frame::classic(standard(), &[1, 2]).unwrap(),
            false,
        );
        for split in 1..packet.len() {
            let mut decoder = Decoder::default();
            assert!(decoder.feed(&packet[..split]).unwrap().is_empty());
            decoder.discard_pending_from(0);
            let mut tail = packet[split..].to_vec();
            tail.extend(&packet);
            let frames = decoder.feed(&tail).unwrap();
            assert_eq!(frames.len(), if channel == 0 { 1 } else { 2 });
        }
    }
}
