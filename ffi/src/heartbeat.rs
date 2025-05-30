// Jackson Coxson

use idevice::{
    IdeviceError, IdeviceService, heartbeat::HeartbeatClient, provider::IdeviceProvider,
};

use crate::{IdeviceErrorCode, IdeviceHandle, RUNTIME, provider::IdeviceProviderHandle};

pub struct HeartbeatClientHandle(pub HeartbeatClient);

/// Automatically creates and connects to Installation Proxy, returning a client handle
///
/// # Arguments
/// * [`provider`] - An IdeviceProvider
/// * [`client`] - On success, will be set to point to a newly allocated InstallationProxyClient handle
///
/// # Returns
/// An error code indicating success or failure
///
/// # Safety
/// `provider` must be a valid pointer to a handle allocated by this library
/// `client` must be a valid, non-null pointer to a location where the handle will be stored
#[unsafe(no_mangle)]
pub unsafe extern "C" fn heartbeat_connect(
    provider: *mut IdeviceProviderHandle,
    client: *mut *mut HeartbeatClientHandle,
) -> IdeviceErrorCode {
    if provider.is_null() || client.is_null() {
        log::error!("Null pointer provided");
        return IdeviceErrorCode::InvalidArg;
    }

    let res: Result<HeartbeatClient, IdeviceError> = RUNTIME.block_on(async move {
        let provider_ref: &dyn IdeviceProvider = unsafe { &*(*provider).0 };
        // Connect using the reference
        HeartbeatClient::connect(provider_ref).await
    });

    match res {
        Ok(r) => {
            let boxed = Box::new(HeartbeatClientHandle(r));
            unsafe { *client = Box::into_raw(boxed) };
            IdeviceErrorCode::IdeviceSuccess
        }
        Err(e) => {
            // If connection failed, the provider_box was already forgotten,
            // so we need to reconstruct it to avoid leak
            let _ = unsafe { Box::from_raw(provider) };
            e.into()
        }
    }
}

/// Automatically creates and connects to Installation Proxy, returning a client handle
///
/// # Arguments
/// * [`socket`] - An IdeviceSocket handle
/// * [`client`] - On success, will be set to point to a newly allocated InstallationProxyClient handle
///
/// # Returns
/// An error code indicating success or failure
///
/// # Safety
/// `socket` must be a valid pointer to a handle allocated by this library. The socket is consumed,
/// and may not be used again.
/// `client` must be a valid, non-null pointer to a location where the handle will be stored
#[unsafe(no_mangle)]
pub unsafe extern "C" fn heartbeat_new(
    socket: *mut IdeviceHandle,
    client: *mut *mut HeartbeatClientHandle,
) -> IdeviceErrorCode {
    if socket.is_null() || client.is_null() {
        return IdeviceErrorCode::InvalidArg;
    }
    let socket = unsafe { Box::from_raw(socket) }.0;
    let r = HeartbeatClient::new(socket);
    let boxed = Box::new(HeartbeatClientHandle(r));
    unsafe { *client = Box::into_raw(boxed) };
    IdeviceErrorCode::IdeviceSuccess
}

/// Sends a polo to the device
///
/// # Arguments
/// * `client` - A valid HeartbeatClient handle
///
/// # Returns
/// An error code indicating success or failure
///
/// # Safety
/// `client` must be a valid pointer to a handle allocated by this library
#[unsafe(no_mangle)]
pub unsafe extern "C" fn heartbeat_send_polo(
    client: *mut HeartbeatClientHandle,
) -> IdeviceErrorCode {
    if client.is_null() {
        return IdeviceErrorCode::InvalidArg;
    }
    let res: Result<(), IdeviceError> = RUNTIME.block_on(async move {
        let client_ref = unsafe { &mut (*client).0 };
        client_ref.send_polo().await
    });
    match res {
        Ok(_) => IdeviceErrorCode::IdeviceSuccess,
        Err(e) => e.into(),
    }
}

/// Sends a polo to the device
///
/// # Arguments
/// * `client` - A valid HeartbeatClient handle
/// * `interval` - The time to wait for a marco
/// * `new_interval` - A pointer to set the requested marco
///
/// # Returns
/// An error code indicating success or failure.
///
/// # Safety
/// `client` must be a valid pointer to a handle allocated by this library
#[unsafe(no_mangle)]
pub unsafe extern "C" fn heartbeat_get_marco(
    client: *mut HeartbeatClientHandle,
    interval: u64,
    new_interval: *mut u64,
) -> IdeviceErrorCode {
    if client.is_null() || new_interval.is_null() {
        return IdeviceErrorCode::InvalidArg;
    }
    let res: Result<u64, IdeviceError> = RUNTIME.block_on(async move {
        let client_ref = unsafe { &mut (*client).0 };
        client_ref.get_marco(interval).await
    });
    match res {
        Ok(n) => {
            unsafe { *new_interval = n };
            IdeviceErrorCode::IdeviceSuccess
        }
        Err(e) => e.into(),
    }
}

/// Frees a handle
///
/// # Arguments
/// * [`handle`] - The handle to free
///
/// # Safety
/// `handle` must be a valid pointer to the handle that was allocated by this library,
/// or NULL (in which case this function does nothing)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn heartbeat_client_free(handle: *mut HeartbeatClientHandle) {
    if !handle.is_null() {
        log::debug!("Freeing installation_proxy_client");
        let _ = unsafe { Box::from_raw(handle) };
    }
}
