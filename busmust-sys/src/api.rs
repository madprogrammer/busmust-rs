use types::*;
use std::os::raw::{c_char, c_int, c_ushort, c_void};

#[cfg(target_arch = "x86_64")]
#[link(name = "bmapi64")]
#[allow(non_snake_case)]
extern "C" {
    /// Initialize BM API library, this function shall be called before any other API calls and shall only be called once.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_Init() -> BMStatus;

    /// Un-initialize BM API library, this function shall be called after any other API calls and shall only be called once.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_UnInit() -> BMStatus;

    /// Enumerate all connected Busmust device.
    ///
    /// # Arguments
    ///
    /// * `channel_infos`: An array of [BMChannelInfo] structure which holds info of all the enumerated Busmust devices.
    /// * `n_channels`: Number of device channels available, which is also the number of valid entries in `channel_infos`,
    /// this param must be initialized with the maximum length of the `channel_infos` array when calling this function.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_Enumerate(channel_infos: *mut BMChannelInfo, n_channels: *mut c_int) -> BMStatus;

    /// Open the specified CAN device port.
    ///
    /// # Arguments
    ///
    /// * `port`: Index of the port, starting from zero, note this is the index of all enumerated ports.
    ///
    /// returns: Handle to the opened CAN device channel, return NULL if failed to open the specified port.
    ///
    pub fn BM_OpenCan(port: c_ushort) -> *const c_void;

    /// Open the specified device port using given configuration.
    ///
    /// # Arguments
    ///
    /// * `handle`: Handle to the opened device channel.
    /// * `channel_info`: Info of the device channel to open, usually the info is filled by [BM_Enumerate].
    /// * `mode`: CAN operation mode option of the opened channel, see [BMCanMode] for details.
    /// * `term`: Terminal resistor option of the opened channel, see [BMTerminalResistor] for details.
    /// * `bit_rate`: Bitrate option of the opened channel, see [BMBitrate] for details.
    /// * `rx_filter_list`: CAN acceptance filters option of the opened channel, see [BMRxFilter] for details.
    /// * `rc_filter_count`: Number of acceptance filters, usually there could be up to 2 filters.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_OpenEx(handle: *mut *mut c_void, channel_info: *const BMChannelInfo,
                     mode: BMCanMode, term: BMTerminalResistor, bit_rate: *const BMBitrate,
                     rx_filter_list: *const BMRxFilter, rc_filter_count: c_int) -> BMStatus;

    /// Close an opened channel.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to be closed.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_Close(channel_handle: *const c_void) -> BMStatus;

    /// Reset an opened channel.
    /// The configuration options will not lost when the channel is reset, so [BM_Reset] is basically identical to [BM_Close] and then [BM_OpenEx].
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to be reset.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_Reset(channel_handle: *const c_void) -> BMStatus;

    /// Activate an opened channel, and thus goes on bus for the selected port and channels.
    /// At this point, the user can transmit and receive messages on the bus.
    /// Channel is default to be activated after [BM_OpenEx] is called.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to be activated.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_Activate(channel_handle: *const c_void) -> BMStatus;

    /// Deactivate an opened channel, and thus the selected channels goes off the bus and stay in BUSOFF state until re-activation.
    /// Any call to [BM_Write] or [BM_Read] will return [BMStatus::BusOff] immediately if the channel is deactivated.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to be deactivated.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_Deactivate(channel_handle: *const c_void) -> BMStatus;

    /// Clear TX&RX message buffer of an opened channel.
    /// This function is available since BM API 1.3, hardware status will not be changed when clearing buffer.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to be cleared.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_ClearBuffer(channel_handle: *const c_void) -> BMStatus;

    /// Read a message/event out of the given channel.
    /// This function is non-blocked, and thus will return [BMStatus::ReceiveBufferEmpty] if no message is received.
    /// Please use notifications to wait for RX events and then read message/event out of BM API internal RX buffer, otherwise you could also poll the device periodically.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to read from.
    /// * `data`: A caller-allocated buffer to hold the message/event output, see [BMData] for details.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_Read(channel_handle: *const c_void, data: *const BMData) -> BMStatus;


    /// Read multiple messages/events out of the given channel.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to read from.
    /// * `data`: A caller-allocated buffer to hold the messages/events array output, see [BMData] for details.
    /// * `n_messages`: Number of read messages, user shall initialize this param with the size (in messages) of the data buffer.
    /// * `timeout`: Timeout (in milliseconds) before the message is received successfully from the bus.
    ///              Set any negative number (i.e. -1) to wait infinitely.
    ///              Set 0 if you would like to receive asynchronously: read from BM API internal buffer and return immediately, use [BM_WaitForNotifications] before reading.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_ReadMultiple(channel_handle: *const c_void, data: *const BMData, n_messages: *mut c_int, timeout: c_int) -> BMStatus;

    /// Read data block using ISO-TP protocol.
    /// This API enables rapid transmission using ISO-TP without app intervention.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to read from.
    /// * `data`: A caller-allocated buffer to hold the data.
    /// * `n_bytes`: Length of the received data block, in bytes. Caller must initialize this argument with the size of the caller-allocated buffer.
    /// * `timeout`: Timeout (in milliseconds) before the message is received successfully from the bus.
    ///              Set any negative number (i.e. -1) to wait infinitely.
    ///              Set 0 if you would like to receive asynchronously: read from BM API internal buffer and return immediately, use [BM_WaitForNotifications] before reading.
    /// * `config`: ISO-TP configuration used by current transfer.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_ReadIsotp(channel_handle: *const c_void, data: *const u8, n_bytes: *mut c_int, timeout: c_int, config: *const BMIsotpConfig) -> BMStatus;

    /// Read CAN message out of the given channel.
    /// Note this function is a simple wrapper of [BM_Read], see [BM_Read] for details.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to read from.
    /// * `msg`: A caller-allocated buffer to hold the CAN message output, see [BMCanMessage] for details.
    /// * `channel`: The source channel ID from which the message is received, starting from zero, could be NULL if not required.
    /// * `timestamp`: The device local high precision timestamp in microseconds, when the message is physically received on the CAN bus, could be NULL if not required.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_ReadCanMessage(channel_handle: *const c_void, msg: *mut BMCanMessage, channel: *mut c_int, timestamp: *mut c_int) -> BMStatus;

    /// Read multiple CAN messages out of the given channel.
    /// This function is non-blocked, and thus will return [BMStatus::ReceiveBufferEmpty] if not all messages are received.
    /// Please use notifications to wait for RX events and then read message/event out of BM API internal RX buffer, otherwise you could also poll the device periodically.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to read from.
    /// * `msgs`: A caller-allocated buffer to hold the CAN message array output, see [BMCanMessage] for details.
    /// * `n_messages`: Number of read messages, user shall initialize this param with the size (in messages) of the data buffer.
    /// * `timeout`: Timeout (in milliseconds) before the message is received successfully from the bus.
    ///              Set any negative number (i.e. -1) to wait infinitely.
    ///              Set 0 if you would like to receive asynchronously: read from BM API internal buffer and return immediately, use [BM_WaitForNotifications] before reading.
    /// * `channels`: The source channel ID from which the message is received, starting from zero, could be NULL if not required.
    /// * `timestamps`: The device local high precision timestamp array in microseconds, when the message is physically transmitted on the CAN bus, could be NULL if not required.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_ReadMultipleCanMessage(channel_handle: *const c_void, msgs: *mut BMCanMessage, n_messages: *mut c_int, timeout: c_int, channels: *mut c_int, timestamps: *mut c_int) -> BMStatus;

    /// Write a message/event to the given channel.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to write to.
    /// * `data`: A caller-allocated buffer to hold the message/event input, see [BMData] for details.
    /// * `timeout`: Timeout (in milliseconds) before the message is transmitted successfully to the bus.
    ///              Set any negative number (i.e. -1) to wait infinitely.
    ///              Set 0 if you would like to transmit asynchronously: put to BM API internal buffer and return immediately, then receive `TXCMPLT` event over [BM_Read] later.
    /// * `timestamp`: The device local high precision timestamp in microseconds, when the message is physically transmitted on the CAN bus, could be NULL if not required.
    ///
    /// returns: BMStatus
    ///
    pub fn BM_Write(channel_handle: *const c_void, data: *const BMData, timeout: c_int, timestamp: *mut c_int) -> BMStatus;

    /// Write multiple messages/events to the given channel.
    /// This function is allowed to be called from multiple threads since BM API 1.3.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to write to.
    /// * `msgs`: A caller-allocated buffer to hold the messages/events array input, see [BMData] for details.
    /// * `n_messages` Number of written messages, user shall initialize this param with the size (in messages) of the data buffer.
    /// * `timeout`: Timeout (in milliseconds) before the message is transmitted successfully to the bus.
    ///              Set any negative number (i.e. -1) to wait infinitely.
    ///              Set 0 if you would like to transmit asynchronously: put to BM API internal buffer and return immediately, then receive `TXCMPLT` event over [BM_Read] later.
    /// * `timestamp`: The device local high precision timestamp array in microseconds, when the message is physically transmitted on the CAN bus, could be NULL if not required.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_WriteMultiple(channel_handle: *const c_void, msgs: *const BMData, n_messages: *mut c_int, timeout: c_int, timestamps: *mut c_int) -> BMStatus;


    /// Write data block using ISO-TP protocol.
    /// This API enables rapid transmission using ISO-TP without app intervention.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to write to.
    /// * `data`: A caller-allocated buffer to hold the data to be sent.
    /// * `n_bytes`: Length of the data block, in bytes.
    /// * `timeout`: Timeout (in milliseconds) before any message segment is transmitted successfully to the bus.
    ///              Note this is only for bus level timeout waiting for CAN ACK, for setting ISO-TP protocol timeouts, see [BMIsotpConfig].
    /// * `config`: ISO-TP configuration used by current transfer.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_WriteIsotp(channel_handle: *const c_void, data: *const u8, n_bytes: c_int, timeout: c_int, config: *const BMIsotpConfig) -> BMStatus;

    /// Write CAN message to the given channel.
    /// Note this function is a simple wrapper to [BM_Write], see [BM_Write] for details.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to write to.
    /// * `msg`: A caller-allocated buffer to hold the CAN message output, see [BMCanMessage] for details.
    /// * `reserved`: The target channel ID to which the message is transmitted, starting from zero. This parameter is reserved for future, always 0 now.
    /// * `timeout`: Timeout (in milliseconds) before the message is transmitted successfully to the bus.
    ///              Set any negative number (i.e. -1) to wait infinitely.
    ///              Set 0 if you would like to transmit asynchronously: put to BM API internal buffer and return immediately, then receive `TXCMPLT` event over [BM_Read] later.
    /// * `timestamp`: The device local high precision timestamp in microseconds, when the message is physically transmitted on the CAN bus, could be NULL if not required.
    ///
    /// returns: BMStatus
    ///
    pub fn BM_WriteCanMessage(channel_handle: *const c_void, msg: *const BMCanMessage, reserved: c_int, timeout: c_int, timestamp: *mut c_int) -> BMStatus;

    /// Write multiple CAN messages to the given channel.
    /// This function is allowed to be called from multiple threads since BM API 1.3.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to write to.
    /// * `msgs`: A caller-allocated buffer to hold the CAN message array input, see [BMCanMessage] for details.
    /// * `n_messages`: Number of written messages, user shall initialize this param with the size (in messages) of the data buffer.
    /// * `reserved`: The target channel ID to which the message is transmitted, starting from zero. This parameter is reserved for future, set to NULL.
    /// * `timeout`: Timeout (in milliseconds) before the message is transmitted successfully to the bus.
    ///              Set any negative number (i.e. -1) to wait infinitely.
    ///              Set 0 if you would like to transmit asynchronously: put to BM API internal buffer and return immediately, then receive `TXCMPLT` event over [BM_Read] later.
    /// * `timestamp`: The device local high precision timestamp array in microseconds, when the message is physically transmitted on the CAN bus, could be NULL if not required.
    ///
    /// returns: BMStatus
    ///
    pub fn BM_WriteMultipleCanMessage(channel_handle: *const c_void, msgs: *const BMCanMessage, n_messages: *mut c_int, reserved: c_int, timeout: c_int, timestamps: *mut c_int) -> BMStatus;

    /// Get current CAN status of the given channel.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to operate on.
    /// * `status_info`: Detailed information of current CAN status, see [BMCanStatusInfo] for details.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_GetStatus(channel_handle: *const c_void, status_info: *mut BMCanStatusInfo) -> BMStatus;

    /// Get current local high precision device timestamp, in microseconds.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to operate on.
    /// * `timestamp`: Timestamp value.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_GetTimestamp(channel_handle: *const c_void, timestamp: *mut c_int) -> BMStatus;

    /// Set CAN mode option of the given channel.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to operate on.
    /// * `mode`: Expected CAN mode, see [BMCanMode] for details.
    ///
    pub fn BM_SetCanMode(channel_handle: *const c_void, mode: BMCanMode) -> BMStatus;

    /// Set terminal resistor option of the given channel.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to operate on.
    /// * `tres`: Expected terminal resistor value, see [BMTerminalResistor] for details.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_SetTerminalRegister(channel_handle: *const c_void, tres: BMTerminalResistor) -> BMStatus;

    /// Set bitrate option of the given channel.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to operate on.
    /// * `bitrate`: Expected bitrate, see [BMBitrate] for details.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_SetBitrate(channel_handle: *const c_void, bitrate: *const BMBitrate) -> BMStatus;

    /// Set TX tasks option of the given channel.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to operate on.
    /// * `tx_tasks`: An array of TX task information, see [BMTxTask] for details.
    /// * `n_tx_tasks`: Number of valid TX tasks in the array.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_SetTxTasks(channel_handle: *const c_void, tx_tasks: *const BMTxTask, n_tx_tasks: c_int) -> BMStatus;

    /// Set RX filters option of the given channel.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel to operate on.
    /// * `rx_filters`: An array of RX filter information, see [BMRxFilter] for details.
    /// * `n_rx_filters`: Number of valid RX filters in the array.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_SetRxFilters(channel_handle: *const c_void, rx_filters: *const BMRxFilter, n_rx_filters: c_int) -> BMStatus;

    /// Get the platform/OS independent notification handle for the given channel, so that the application could wait for notifications later.
    ///
    /// # Arguments
    ///
    /// * `channel_handle`: Handle to the channel that owns the notification handle.
    /// * `notification`: The platform/OS independent notification handle.
    ///
    /// returns: [BMStatus]
    ///
    pub fn BM_GetNotification(channel_handle: *const c_void, notification: *mut *mut c_void) -> BMStatus;


    /// A platform/OS independent implementation to wait for single/multiple notification handles.
    ///
    /// # Arguments
    ///
    /// * `handles`: An array of channel notification handles.
    /// * `n_handles`: Number of valid notification handles.
    /// * `timeout_ms`: This function will block the current thread for at most `timeout_ms` milliseconds if no notification is received.
    ///
    /// returns: This function returns an index into the handles array for which a notification has been received.
    ///
    pub fn BM_WaitForNotifications(handles: *const *const c_void, n_handles: c_int, timeout_ms: c_int) -> c_int;


    /// Translate error code to string, this is a helper function to ease application programming.
    ///
    /// # Arguments
    ///
    /// * `status`: The error code to be translated.
    /// * `buffer`: A caller-allocated string buffer to hold the translated string.
    /// * `length`: Number in bytes of the string buffer.
    /// * `reserved`: Reserved.
    ///
    pub fn BM_GetErrorText(status: BMStatus, buffer: *mut c_char, length: usize, reserved: c_ushort);

    /// Translate data (i.e. CAN message) to string, this is a helper function to ease application programming.
    ///
    /// # Arguments
    ///
    /// * `data`: The message data to be translated.
    /// * `buffer`: A caller-allocated string buffer to hold the translated string.
    /// * `length`: Number in bytes of the string buffer.
    /// * `reserved`: Reserved.
    ///
    pub fn BM_GetDataText(data: *const BMData, buffer: *mut c_char, length: usize, reserved: c_ushort);

    /// Set library log level.
    ///
    /// # Arguments
    ///
    /// * `level`: Target log level, all messages equal to or less than this level will be printed on debug console.
    ///
    pub fn BM_SetLogLevel(level: BMLogLevel);

    /// Get library log level.
    ///
    /// returns: Current log level, all messages equal to or less than this level are currently printed on debug console.
    ///
    pub fn BM_GetLogLevel() -> BMLogLevel;
}
