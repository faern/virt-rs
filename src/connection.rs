use crate::error::{Error, VirtError};
use virt_sys::{virConnectClose, virConnectOpen, virConnectPtr};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::ptr;

pub struct Connection(virConnectPtr);

impl Connection {
    /// Tries to open a connection to the hypervisor at externally defined URI.
    /// If the environment variable `LIBVIRT_DEFAULT_URI` is set, then it will be used.
    /// Otherwise if the client configuration file has the "uri_default" parameter set,
    /// then it will be used. Finally probing will be done to determine a suitable default driver
    /// to activate. This involves trying each hypervisor in turn until one successfully opens.
    pub fn open_default() -> Result<Self, Error> {
        unsafe { Self::open_internal(ptr::null()).map_err(Error::VirtError) }
    }

    /// Tries to open a connection to the hypervisor at the given URI.
    pub fn open_uri(uri: &str) -> Result<Self, Error> {
        let c_uri = CString::new(uri).map_err(Error::InvalidUri)?;
        Self::open_c_uri(&c_uri).map_err(Error::VirtError)
    }

    /// Tries to open a connection to the hypervisor with a C string URI directly.
    /// This method avoids the possible `InvalidUri` error that `open_uri` can run into.
    pub fn open_c_uri(uri: &CStr) -> Result<Self, VirtError> {
        unsafe { Self::open_internal(uri.as_ptr()) }
    }

    unsafe fn open_internal(uri_ptr: *const c_char) -> Result<Self, VirtError> {
        let connect_ptr = virConnectOpen(uri_ptr);
        if connect_ptr.is_null() {
            Err(VirtError::last_virt_error())
        } else {
            Ok(Self(connect_ptr))
        }
    }

    /// Returns an instance backed by the given pointer.
    ///
    /// # Safety
    ///
    /// This method assumes it can take over ownership of the connection behind the pointer.
    /// The instance will call `virConnectClose` on the given pointer when it goes out of scope.
    pub unsafe fn from_ptr(ptr: virConnectPtr) -> Self {
        Self(ptr)
    }

    /// Returns a pointer to the underlying libvirt struct. The pointer is valid
    /// as long as this connection instance is in scope.
    pub fn as_ptr(&self) -> virConnectPtr {
        self.0
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // FIXME: check for negative return value
        let _ = unsafe { virConnectClose(self.0) };
    }
}
