use std::ffi::c_void;
use super::Error;
use std::{mem, ptr};
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_int};
use call::cvt_r;
use ffi::*;
use anyhow::Result;


use util::StringExt;

pub fn desc_from_error(err: &Error) -> String {
    unsafe {
        match err {
            Error::BusmustError(status) => {
                let mut desc: [c_char; BM_ERROR_DESC_MAX_SIZE + 1] = mem::zeroed();
                BM_GetErrorText(
                    *status, desc[..].as_mut_ptr(), desc.len(), 0);
                String::from_slice(&desc[..])
            }
        }
    }
}

pub fn initialize() -> Result<()> {
    unsafe {
        cvt_r(BM_Init())?;
    }
    Ok(())
}

pub fn terminate() -> Result<()> {
    unsafe {
        cvt_r(BM_UnInit())?;
    }
    Ok(())
}

pub struct Device (BMChannelInfo, Option<*const c_void>, Option<*const c_void>);

impl Device {
    /// Open the device channel with default parameters.
    ///
    /// # Examples
    ///
    /// ```
    /// device.open.unwrap()
    /// ```
    pub fn open(&mut self) -> Result<()> {
        unsafe {
            let handle = BM_OpenCan(self.0.port);
            if !handle.is_null() {
                self.1 = Some(handle);
                self.get_notification()
            } else {
                Err(Error::BusmustError(BMStatus::InvalidOperation).into())
            }
        }
    }

    pub fn open_ex(&mut self) -> Result<()> {
        let bitrate = BMBitrate::builder().build();

        unsafe {
            let mut handle: *mut c_void = ptr::null_mut();
            let handle_ptr: *mut *mut c_void = &mut handle;

            cvt_r(BM_OpenEx(
                handle_ptr,
                &self.0,
                BMCanMode::Normal,
                BMTerminalResistor::Enabled120,
                &bitrate,
                0 as *const BMRxFilter,
                0)).and_then(|_| {
                    self.1 = Some(handle.cast_const());
                    self.get_notification()
                })
        }
    }


    /// Set bitrate option of the opened channel.
    ///
    /// # Arguments
    ///
    /// * `bitrate`: Expected bitrate, see [BMBitrate] for details.
    ///
    /// returns: [Result<()>]
    ///
    /// # Examples
    ///
    /// ```
    /// use busmust_sys::BMBitrate;
    ///
    /// device.set_bitrate(BMBitrate::builder().bitrate(250).build()).unwrap();
    /// ```
    pub fn set_bitrate(&self, bitrate: BMBitrate) -> Result<()> {
        unsafe {
            cvt_r(BM_SetBitrate(self.1.expect("not opened"), &bitrate))
        }
    }

    /// Get the string name of the device
    pub fn name(&self) -> String {
        String::from_slice(&self.0.name[..])
    }

    /// Get device serial number as string
    pub fn serial_number(&self) -> String {
        String::from_utf8_lossy(&self.0.sn[..]).to_string()
    }

    /// Get the unique ID of the device as string
    pub fn unique_id(&self) -> String {
        String::from_utf8_lossy(&self.0.uid[..]).to_string()
    }

    pub fn version(&self) -> Vec<u8> {
        self.0.version.to_vec()
    }

    /// Get the USB vendor ID of the device
    pub fn vendor_id(&self) -> u16 {
        self.0.vid
    }

    /// Get the USB product ID of the device
    pub fn product_id(&self) -> u16 {
        self.0.pid
    }

    /// Get the index of device port (for devices with multiple ports)
    pub fn port(&self) -> u16 {
        self.0.port
    }

    /// Get device capabilities
    pub fn caps(&self) -> BMCapability {
        self.0.cap
    }

    /// Close previously opened channel
    pub fn close(&self) -> Result<()> {
        unsafe {
            cvt_r(BM_Close(self.1.expect("not opened")))
        }
    }

    /// Reset the opened channel.
    /// The configuration options will not be lost when the channel is reset, so [Device::reset] is basically identical to [Device::close] and [Device::open].
    pub fn reset(&self) -> Result<()> {
        unsafe {
            cvt_r(BM_Reset(self.1.expect("not opened")))
        }
    }

    /// Activate the opened channel. After that the user can transmit and receive messages on the bus.
    /// Channel will be active by default after [Device::open_ex] is called.
    pub fn activate(&self) -> Result<()> {
        unsafe {
            cvt_r(BM_Activate(self.1.expect("not opened")))
        }
    }

    /// Deactivate the opened channel. The channel will stay in BUS OFF state until re-activation.
    /// Any read/write call will raise a [BMStatus::BusOff] error immediately if the channel is deactivated.
    pub fn deactivate(&self) -> Result<()> {
        unsafe {
            cvt_r(BM_Deactivate(self.1.expect("not opened")))
        }
    }

    /// Clear internal TX & RX message buffer of the opened channel.
    pub fn clear_buffer(&self) -> Result<()> {
        unsafe {
            cvt_r(BM_ClearBuffer(self.1.expect("not opened")))
        }
    }

    /// Get current CAN status of the opened channel.
    pub fn get_status_info(&self) -> Result<BMCanStatusInfo> {
        unsafe {
            let mut status_info = MaybeUninit::<BMCanStatusInfo>::uninit();
            cvt_r(BM_GetStatus(
                self.1.expect("not opened"),
                status_info.as_mut_ptr(),
            )).and_then(|_| Ok(status_info.assume_init()))
        }
    }

    /// Get current value of tge high precision device timestamp, in microseconds.
    pub fn get_timestamp(&self) -> Result<u32> {
        unsafe {
            let mut timestamp = 0;
            cvt_r(BM_GetTimestamp(
                self.1.expect("not opened"),
                &mut timestamp
            )).and_then(|_| Ok(timestamp as u32))
        }
    }

    /// Set terminal resistor option of the opened channel.
    ///
    /// # Arguments
    ///
    /// * `value`: Expected terminal resistor value, see [BMTerminalResistor] for details.
    ///
    /// returns: [Result<()>]
    ///
    /// # Examples
    ///
    /// ```
    /// use busmust_sys::BMTerminalResistor;
    ///
    /// device.set_terminal_resistor(BMTerminalResistor::Enabled120).unwrap()
    /// ```
    pub fn set_terminal_resistor(&self, value: BMTerminalResistor) -> Result<()> {
        unsafe {
            cvt_r(BM_SetTerminalRegister(
                self.1.expect("not opened"),
                value
            ))
        }
    }

    /// Set CAN mode option of the opened channel.
    ///
    /// # Arguments
    ///
    /// * `mode`: Expected CAN mode, see [BMCanMode] for details.
    ///
    /// returns: [Result<()>]
    ///
    /// # Examples
    ///
    /// ```
    /// use busmust_sys::BMCanMode;
    ///
    /// device.set_can_mode(BMCanMode::Normal).unwrap()
    /// ```
    pub fn set_can_mode(&self, mode: BMCanMode) -> Result<()> {
        unsafe {
            cvt_r(BM_SetCanMode(
                self.1.expect("not opened"),
                mode
            ))
        }
    }

    /// Write a message/event to the opened channel.
    ///
    /// # Arguments
    ///
    /// * `message`: A [BMData] struct which represents the message.
    /// * `timeout`: Optional timeout in `ms`, use `-1` to wait indefinitely or `0` to send asynchronously.
    ///
    /// returns: The device local high precision timestamp in microseconds, when the message is physically transmitted on the CAN bus, or error.
    ///
    /// # Examples
    ///
    /// ```
    /// use busmust_sys::{BMCanMessage, BMData};
    ///
    /// let msg = BMCanMessage::builder()
    ///  .sid(0x123)
    ///  .payload(vec![1, 2, 3, 4, 5, 6, 7, 8])
    ///  .build();
    ///
    /// let data = BMData::builder()
    ///  .can_message(msg)
    ///  .build();
    ///
    /// device.write(data, Some(1000)).unwrap();
    /// ```
    pub fn write(&self, message: BMData, timeout: Option<i32>) -> Result<u32> {
        unsafe {
            let mut timestamp = 0;
            cvt_r(BM_Write(
                self.1.expect("not opened"),
                &message,
                timeout.unwrap_or_default(),
                &mut timestamp
            )).and_then(|_| Ok(timestamp as u32))
        }
    }

    /// Write multiple message/event to the opened channel.
    ///
    /// # Arguments
    ///
    /// * `messages`: An array of messages/events to be sent.
    /// * `timeout`: Optional timeout in `ms`, use `-1` to wait indefinitely or `0` to send asynchronously.
    ///
    /// returns: An array of device local high precision timestamps in microseconds, when the messages are physically transmitted, or error.
    ///
    pub fn write_multiple(&self, messages: Vec<BMData>, timeout: Option<i32>) -> Result<Vec<u32>> {
        unsafe {
            let mut n_messages = 0;

            let mut timestamps = Vec::<u32>::with_capacity(messages.len());
            timestamps.reserve_exact(messages.len());

            cvt_r(BM_WriteMultiple(
                self.1.expect("not opened"),
                messages.as_ptr(),
                &mut n_messages,
                timeout.unwrap_or_default(),
                timestamps.as_mut_ptr() as *mut c_int
            )).and_then(|_| {
                timestamps.truncate(n_messages as usize);
                Ok(timestamps)
            })
        }
    }

    /// Write single CAN message to the opened channel.
    ///
    /// # Arguments
    ///
    /// * `message`: A [BMCanMessage] struct which represents the message.
    /// * `timeout`: Optional timeout in `ms`, use `-1` to wait indefinitely or `0` to send asynchronously.
    ///
    /// returns: The device local high precision timestamp in microseconds, when the message is physically transmitted on the CAN bus, or error.
    ///
    /// # Examples
    ///
    /// ```
    /// use busmust_sys::BMCanMessage;
    ///
    /// let msg = BMCanMessage::builder()
    ///     .sid(0x123)
    ///     .payload(vec![1, 2, 3, 4, 5, 6, 7, 8])
    ///     .build();
    /// device.write_can_message(msg, Some(1000)).unwrap();
    /// ```
    pub fn write_can_message(&self, message: BMCanMessage, timeout: Option<i32>) -> Result<u32> {
        unsafe {
            let mut timestamp = 0;
            cvt_r(BM_WriteCanMessage(
                self.1.expect("not opened"),
                &message,
                0,
                timeout.unwrap_or_default(),
                &mut timestamp
            )).and_then(|_| Ok(timestamp as u32))
        }
    }

    /// Write multiple CAN messages to the opened channel.
    ///
    /// # Arguments
    ///
    /// * `messages`: An array of messages to be sent.
    /// * `timeout`: Optional timeout in `ms`, use `-1` to wait indefinitely or `0` to send asynchronously.
    ///
    /// returns: An array of device local high precision timestamps in microseconds, when the messages are physically transmitted on the CAN bus, or error.
    ///
    /// # Examples
    ///
    /// ```
    /// use busmust_sys::BMCanMessage;
    ///
    /// let msg = BMCanMessage::builder()
    ///     .sid(0x123)
    ///     .payload(vec![1, 2, 3, 4, 5, 6, 7, 8])
    ///     .build();
    /// device.write_can_messages(vec![msg, msg, msg], Some(1000)).unwrap();
    /// ```
    pub fn write_can_messages(&self, messages: Vec<BMCanMessage>, timeout: Option<i32>) -> Result<Vec<u32>> {
        unsafe {
            let mut n_messages = 0;

            let mut timestamps = Vec::<u32>::with_capacity(messages.len());
            timestamps.reserve_exact(messages.len());

            cvt_r(BM_WriteMultipleCanMessage(
                self.1.expect("not opened"),
                messages.as_ptr(),
                &mut n_messages,
                0,
                timeout.unwrap_or_default(),
                timestamps.as_mut_ptr() as *mut c_int
            )).and_then(|_| {
                timestamps.truncate(n_messages as usize);
                Ok(timestamps)
            })
        }
    }

    /// Read a message/event out of the opened channel.
    /// This function is non-blocking, and thus will raise [BMStatus::ReceiveBufferEmpty] if no message is received.
    /// Please use notifications to wait for RX events and then read message/event out of the internal RX buffer
    /// You could also poll the device periodically./
    ///
    /// returns: [`Result<BMData>`]
    ///
    pub fn read(&self) -> Result<BMData> {
        unsafe {
            let mut data = MaybeUninit::<BMData>::uninit();
            cvt_r(BM_Read(self.1.expect("not opened"), data.as_mut_ptr()))
                .and_then(|_| Ok(data.assume_init()))
        }
    }

    /// Read multiple messages/events out of the given channel.
    ///
    /// # Arguments
    ///
    /// * `n_messages`: Number of messages to read.
    /// * `timeout`: Optional timeout in `ms`, use `-1` to wait indefinitely or `0` to receive asynchronously.
    ///
    /// returns: [`Result<Vec<BMData>>`]
    ///
    /// # Examples
    ///
    /// ```
    /// let messages = device.read_multiple(10, Some(1000)).unwrap();
    /// ```
    pub fn read_multiple(&self, n_messages: usize, timeout: Option<i32>) -> Result<Vec<BMData>> {
        unsafe {
            let mut read_messages = n_messages as c_int;
            let mut messages = Vec::<BMData>::with_capacity(n_messages);

            cvt_r(BM_ReadMultiple(self.1.expect("not opened"),
                                  messages.as_mut_ptr(),
                                  &mut read_messages,
                                  timeout.unwrap_or_default()
            )).and_then(|_| {
                messages.truncate(read_messages as usize);
                Ok(messages)
            })
        }
    }

    /// Read CAN message out of the opened channel.
    /// This function is non-blocking, use [Device::wait_for_notification] to wait for a message first.
    ///
    /// returns: [`Result<BMCanMessage>`]
    ///
    /// # Examples
    ///
    /// ```
    /// let msg = device.read_can_message().unwrap();
    /// ```
    pub fn read_can_message(&self) -> Result<Option<BMCanMessage>> {
        unsafe {
            let mut message = MaybeUninit::<BMCanMessage>::uninit();
            let result = cvt_r(BM_ReadCanMessage(self.1.expect("not opened"),
                message.as_mut_ptr(),
                ptr::null_mut(),
                ptr::null_mut()
            ));
            if result.is_ok() {
                Ok(Some(message.assume_init()))
            } else {
                result.map(|_|None)
            }
        }
    }

    /// Read multiple CAN messages from the opened channel.
    ///
    /// # Arguments
    ///
    /// * `n_messages`: Read at most `n_messages` messages.
    /// * `timeout`: Maximum time in `ms` to wait until all requested messages are read.
    ///
    /// returns: [`Result<Vec<BMCanMessage>>`]
    ///
    /// # Examples
    ///
    /// ```
    /// let messages = device.read_can_messages(10, Some(1000)).unwrap();
    /// ```
    pub fn read_can_messages(&self, n_messages: usize, timeout: Option<i32>) -> Result<Vec<BMCanMessage>> {
        unsafe {
            let mut read_messages = n_messages as c_int;
            let mut messages = Vec::<BMCanMessage>::with_capacity(n_messages);

            cvt_r(BM_ReadMultipleCanMessage(self.1.expect("not opened"),
                messages.as_mut_ptr(),
                &mut read_messages,
                timeout.unwrap_or_default(),
                ptr::null_mut(),
                ptr::null_mut()
            )).and_then(|_| {
                messages.truncate(read_messages as usize);
                Ok(messages)
            })
        }
    }

    /// Wait for event/message notification on the opened channel.
    ///
    /// # Arguments
    ///
    /// * `timeout`: This function will block the current thread for at most `timeout` milliseconds if no notification is received.
    ///
    /// returns: bool
    ///
    /// # Examples
    ///
    /// ```
    /// device.wait_for_notification(Some(1000));
    /// ```
    pub fn wait_for_notification(&self, timeout: Option<u32>) -> bool {
        unsafe {
            let index = BM_WaitForNotifications(
                &self.2.expect("not opened"),
                1,
                timeout.unwrap_or_default() as c_int
            );
            index >= 0
        }
    }

    /// Set library log level. See [BMLogLevel].
    ///
    /// # Arguments
    ///
    /// * `level`: Log level to set. See [BMLogLevel].
    ///
    /// returns: ()
    ///
    /// # Examples
    ///
    /// ```
    /// use busmust_sys::BMLogLevel;
    ///
    /// device.set_log_level(BMLogLevel::Info);
    /// ```
    pub fn set_log_level(&self, level: BMLogLevel) {
        unsafe {
            BM_SetLogLevel(level)
        }
    }

    /// Get library log level. See [BMLogLevel].
    ///
    /// returns: [BMLogLevel]
    ///
    /// # Examples
    ///
    /// ```
    /// device.get_log_level()
    /// ```
    pub fn get_log_level(&self) -> BMLogLevel {
        unsafe {
            BM_GetLogLevel()
        }
    }

    /// Get platform-independent notification handle for opened channel
    fn get_notification(&mut self) -> Result<()> {
        unsafe {
            let mut handle: *mut c_void = ptr::null_mut();
            let handle_ptr: *mut *mut c_void = &mut handle;

            let result = cvt_r(BM_GetNotification(
                self.1.expect("not opened"), handle_ptr));
            self.2 = Some(handle.cast_const());

            result
        }
    }
}

pub struct Devices {
    current: usize,
    count: usize,
    device_infos: Vec<BMChannelInfo>
}

impl Devices {
    fn new(count: usize, infos: Vec<BMChannelInfo>) -> Self {
        Devices { count, current: 0, device_infos: infos }
    }
}

impl Iterator for Devices {
    type Item = Device;

    fn next(&mut self) -> Option<Device> {
        if self.current < self.count {
            let device = Device(self.device_infos[self.current],
                                None, None);
            self.current += 1;

            Some(device)
        } else {
            None
        }
    }
}

pub fn enum_devices() -> Result<Devices> {
    let mut infos = Vec::with_capacity(64);
    let mut count: c_int = infos.capacity() as c_int;

    unsafe {
        cvt_r(BM_Enumerate(infos.as_mut_ptr(), &mut count)).unwrap();
        infos.set_len(count as usize);
    }

    Ok(Devices::new(count as usize, infos))
}
