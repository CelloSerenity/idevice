// Jackson Coxson

use idevice::{IdeviceError, IdeviceService, amfi::AmfiClient, provider::IdeviceProvider};

use crate::{IdeviceErrorCode, IdeviceHandle, RUNTIME, provider::IdeviceProviderHandle};

pub struct AmfiClientHandle(pub AmfiClient);

/// Automatically creates and connects to AMFI service, returning a client handle
///
/// # Arguments
/// * [`provider`] - An IdeviceProvider
/// * [`client`] - On success, will be set to point to a newly allocated AmfiClient handle
///
/// # Returns
/// An error code indicating success or failure
///
/// # Safety
/// `provider` must be a valid pointer to a handle allocated by this library
/// `client` must be a valid, non-null pointer to a location where the handle will be stored
#[unsafe(no_mangle)]
pub unsafe extern "C" fn amfi_connect(
    provider: *mut IdeviceProviderHandle,
    client: *mut *mut AmfiClientHandle,
) -> IdeviceErrorCode {
    if provider.is_null() || client.is_null() {
        log::error!("Null pointer provided");
        return IdeviceErrorCode::InvalidArg;
    }

    let res: Result<AmfiClient, IdeviceError> = RUNTIME.block_on(async move {
        let provider_ref: &dyn IdeviceProvider = unsafe { &*(*provider).0 };

        // Connect using the reference
        AmfiClient::connect(provider_ref).await
    });

    match res {
        Ok(r) => {
            let boxed = Box::new(AmfiClientHandle(r));
            unsafe { *client = Box::into_raw(boxed) };
            IdeviceErrorCode::IdeviceSuccess
        }
        Err(e) => e.into(),
    }
}

/// Automatically creates and connects to AMFI service, returning a client handle
///
/// # Arguments
/// * [`socket`] - An IdeviceSocket handle
/// * [`client`] - On success, will be set to point to a newly allocated AmfiClient handle
///
/// # Returns
/// An error code indicating success or failure
///
/// # Safety
/// `socket` must be a valid pointer to a handle allocated by this library. It is consumed, and
/// should not be used again.
/// `client` must be a valid, non-null pointer to a location where the handle will be stored
#[unsafe(no_mangle)]
pub unsafe extern "C" fn amfi_new(
    socket: *mut IdeviceHandle,
    client: *mut *mut AmfiClientHandle,
) -> IdeviceErrorCode {
    if socket.is_null() || client.is_null() {
        return IdeviceErrorCode::InvalidArg;
    }

    let socket = unsafe { Box::from_raw(socket) }.0;
    let r = AmfiClient::new(socket);
    let boxed = Box::new(AmfiClientHandle(r));
    unsafe { *client = Box::into_raw(boxed) };
    IdeviceErrorCode::IdeviceSuccess
}

/// Shows the option in the settings UI
///
/// # Arguments
/// * `client` - A valid AmfiClient handle
///
/// # Returns
/// An error code indicating success or failure
///
/// # Safety
/// `client` must be a valid pointer to a handle allocated by this library
#[unsafe(no_mangle)]
pub unsafe extern "C" fn amfi_reveal_developer_mode_option_in_ui(
    client: *mut AmfiClientHandle,
) -> IdeviceErrorCode {
    if client.is_null() {
        return IdeviceErrorCode::InvalidArg;
    }

    let res: Result<(), IdeviceError> = RUNTIME.block_on(async move {
        let client_ref = unsafe { &mut (*client).0 };
        client_ref.reveal_developer_mode_option_in_ui().await
    });
    match res {
        Ok(_) => IdeviceErrorCode::IdeviceSuccess,
        Err(e) => e.into(),
    }
}

/// Enables developer mode on the device
///
/// # Arguments
/// * `client` - A valid AmfiClient handle
///
/// # Returns
/// An error code indicating success or failure
///
/// # Safety
/// `client` must be a valid pointer to a handle allocated by this library
#[unsafe(no_mangle)]
pub unsafe extern "C" fn amfi_enable_developer_mode(
    client: *mut AmfiClientHandle,
) -> IdeviceErrorCode {
    if client.is_null() {
        return IdeviceErrorCode::InvalidArg;
    }

    let res: Result<(), IdeviceError> = RUNTIME.block_on(async move {
        let client_ref = unsafe { &mut (*client).0 };
        client_ref.enable_developer_mode().await
    });
    match res {
        Ok(_) => IdeviceErrorCode::IdeviceSuccess,
        Err(e) => e.into(),
    }
}

/// Accepts developer mode on the device
///
/// # Arguments
/// * `client` - A valid AmfiClient handle
///
/// # Returns
/// An error code indicating success or failure
///
/// # Safety
/// `client` must be a valid pointer to a handle allocated by this library
#[unsafe(no_mangle)]
pub unsafe extern "C" fn amfi_accept_developer_mode(
    client: *mut AmfiClientHandle,
) -> IdeviceErrorCode {
    if client.is_null() {
        return IdeviceErrorCode::InvalidArg;
    }

    let res: Result<(), IdeviceError> = RUNTIME.block_on(async move {
        let client_ref = unsafe { &mut (*client).0 };
        client_ref.accept_developer_mode().await
    });
    match res {
        Ok(_) => IdeviceErrorCode::IdeviceSuccess,
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
pub unsafe extern "C" fn amfi_client_free(handle: *mut AmfiClientHandle) {
    if !handle.is_null() {
        log::debug!("Freeing AmfiClient handle");
        let _ = unsafe { Box::from_raw(handle) };
    }
}
