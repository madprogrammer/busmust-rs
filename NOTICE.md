BUSMUST Rust driver — license and attribution

The native USB driver is distributed under the GNU General Public License,
version 2 or (at your option) any later version. See LICENSE-GPL.

The Rust adaptation was made on 2026-10-07. It replaces the original vendor-SDK
bindings with a Rust API, nusb transport, explicit wire codec, and legacy bus-off
recovery, drawing on these GPL-2.0-or-later implementations:

* python-can-busmust: protocol.py, transport.py, and bus.py
  https://github.com/madprogrammer/python-can-busmust
  Reference revision: a4efb7d866f73f6bb84e22c93329d22382a8cb06
* bmsocketcan: bmcan_proto.c, bmcan_usb.c, and bmcan_netdev.c
  https://github.com/busmust/bmsocketcan
  Reference revision: 70919fbc2a1be146498ca0df941d1d8a0a93c74b
  Copyright (C) 2026 Busmust Tech Co.,Ltd
  SPDX-FileCopyrightText: 2026 Busmust Tech Co.,Ltd

The Rust version adds ownership-based resource management, validated frame and
configuration types, incremental decoding, host-side filter enforcement, bounded
recovery, and Rust tests. The upstream vendor header inc/bm_usb_def.h has separate
restricted terms; that header is not included or relicensed in this distribution.

The original busmust-rs project was dual-licensed MIT OR Apache-2.0:
Copyright (c) 2018 Dan Glastonbury
Copyright (c) 2023 Sergey Anufrienko
Its license texts and notices are retained in LICENSE-MIT and LICENSE-APACHE.
The MIT option is used for any original material incorporated into this GPL
version. Those historical licenses do not offer an alternative license for the
new native driver as a whole.

Version 0.2.0 mistakenly retained the old MIT OR Apache-2.0 package metadata.
Version 0.2.1 corrects the licensing and supplies these attribution notices.
Previously published crates.io archives are immutable; use 0.2.1 or later.
