use busmust::{ChannelConfig, FdOptions, Frame, Id};

#[test]
fn identifiers_enforce_can_ranges_and_preserve_format() {
    assert!(Id::standard(0x7ff).is_ok());
    assert!(Id::standard(0x800).is_err());
    assert!(Id::extended(0x1fff_ffff).is_ok());
    assert!(Id::extended(0x2000_0000).is_err());
    assert_ne!(Id::standard(1).unwrap(), Id::extended(1).unwrap());
    assert_eq!(Id::extended(1).unwrap().as_raw(), 1);
}

#[test]
fn frame_construction_rejects_invalid_lengths() {
    let id = Id::standard(0).unwrap();
    assert!(Frame::classic(id, &[0; 9]).is_err());
    assert!(Frame::fd(id, &[0; 65], FdOptions::default()).is_err());
    assert!(Frame::remote(id, 9).is_err());
    assert!(Frame::classic(id, &[]).unwrap().is_empty());
    assert!(Frame::fd(id, &[], FdOptions::default()).unwrap().is_fd());
    assert!(Frame::remote(id, 8).unwrap().fd_options().is_none());
}

#[test]
fn bitrates_and_sample_points_are_checked() {
    for bitrate in [0, 123_456, 65_536_000] {
        assert!(ChannelConfig {
            bitrate,
            ..Default::default()
        }
        .validate()
        .is_err());
        assert!(ChannelConfig {
            data_bitrate: bitrate,
            ..Default::default()
        }
        .validate()
        .is_err());
    }
    for sample_point in [0, 100, 255] {
        assert!(ChannelConfig {
            sample_point,
            ..Default::default()
        }
        .validate()
        .is_err());
        assert!(ChannelConfig {
            data_sample_point: sample_point,
            ..Default::default()
        }
        .validate()
        .is_err());
    }
    assert!(ChannelConfig::default().validate().is_ok());
}

#[test]
fn filter_masks_respect_identifier_width() {
    use busmust::Filter;
    assert!(Filter::new(Id::standard(1).unwrap(), 0x800).is_err());
    assert!(Filter::new(Id::extended(1).unwrap(), 0x2000_0000).is_err());
    assert_eq!(Filter::exact(Id::standard(1).unwrap()).mask(), 0x7ff);
    assert_eq!(Filter::exact(Id::extended(1).unwrap()).mask(), 0x1fff_ffff);
}

#[test]
fn filters_enforce_format_and_flags_even_when_firmware_does_not() {
    use busmust::{Filter, ReceiveFilters};
    let id = Id::standard(0x123).unwrap();
    let remote = Frame::remote(id, 8).unwrap();
    let classic = Frame::classic(id, &[0; 8]).unwrap();
    let fd = Frame::fd(id, &[0; 12], FdOptions::default()).unwrap();
    let brs = Frame::fd(
        id,
        &[0; 12],
        FdOptions {
            bitrate_switch: true,
        },
    )
    .unwrap();
    let filter = Filter::exact(id).with_remote(true);
    assert!(filter.matches(&remote));
    assert!(!filter.matches(&classic));
    let filter = Filter::exact(id).with_fd(true).with_bitrate_switch(true);
    assert!(filter.matches(&brs));
    assert!(!filter.matches(&fd));
    assert!(!filter.matches(&classic));
    assert!(!Filter::exact(Id::extended(0x123).unwrap()).matches(&classic));
    let range = Filter::new(Id::standard(0x12a).unwrap(), 0x7f0).unwrap();
    assert!(range.matches(&classic));
    assert!(ReceiveFilters::Either(range, filter).matches(&classic));
}
