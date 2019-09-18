use crate::error::{Error, VirtError};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_ulong};
use std::ptr;
use virt_sys::{
    virConnectClose, virConnectGetHostname, virConnectGetLibVersion, virConnectGetType,
    virConnectGetURI, virConnectGetVersion, virConnectOpen, virConnectPtr, virConnectRef,
};

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

    /// Returns the system hostname on which the hypervisor is running.
    pub fn hostname(&self) -> Result<String, Error> {
        let hostname_ptr = cvt_null!(unsafe { virConnectGetHostname(self.0) })?;
        let hostname = unsafe { CStr::from_ptr(hostname_ptr) }
            .to_str()
            .map(str::to_owned);
        unsafe {
            libc::free(hostname_ptr as *mut _);
        }
        Ok(hostname.map_err(Error::Utf8Error)?)
    }

    pub fn hostname_cstr(&self) -> Result<CString, VirtError> {
        let hostname_ptr = cvt_null!(unsafe { virConnectGetHostname(self.0) })?;
        let hostname = unsafe {
            CString::from_vec_unchecked(CStr::from_ptr(hostname_ptr).to_bytes().to_vec())
        };
        unsafe {
            libc::free(hostname_ptr as *mut _);
        }
        Ok(hostname)
    }

    pub fn uri(&self) -> Result<String, Error> {
        let uri_ptr = cvt_null!(unsafe { virConnectGetURI(self.0) })?;
        let uri = unsafe { CStr::from_ptr(uri_ptr) }
            .to_str()
            .map(str::to_owned);
        unsafe {
            libc::free(uri_ptr as *mut _);
        }
        Ok(uri.map_err(Error::Utf8Error)?)
    }

    pub fn uri_cstr(&self) -> Result<CString, VirtError> {
        let uri_ptr = cvt_null!(unsafe { virConnectGetURI(self.0) })?;
        let uri =
            unsafe { CString::from_vec_unchecked(CStr::from_ptr(uri_ptr).to_bytes().to_vec()) };
        unsafe {
            libc::free(uri_ptr as *mut _);
        }
        Ok(uri)
    }

    /// Returns the name of the Hypervisor driver used. This is merely the driver name;
    /// for example, both KVM and QEMU guests are serviced by the driver for the qemu:// URI,
    /// so a return of "QEMU" does not indicate whether KVM acceleration is present
    pub fn hypervisor_type(&self) -> Result<&'static str, Error> {
        self.hypervisor_type_cstr()
            .map_err(Error::VirtError)
            .and_then(|hypervisor_type| hypervisor_type.to_str().map_err(Error::Utf8Error))
    }

    /// See [`hypervisor_type`].
    pub fn hypervisor_type_cstr(&self) -> Result<&'static CStr, VirtError> {
        let ptr = cvt_null!(unsafe { virConnectGetType(self.0) })?;
        Ok(unsafe { CStr::from_ptr(ptr) })
    }

    /// Returns the version of libvirt used by the hypervisor this connection is connected to.
    pub fn lib_version(&self) -> Result<crate::version::Version, VirtError> {
        let mut lib_ver: c_ulong = 0;
        match unsafe { virConnectGetLibVersion(self.0, &mut lib_ver) } {
            -1 => Err(VirtError::last_virt_error()),
            0 => Ok(crate::version::Version::from(lib_ver)),
            i => panic!(
                "Unexpected return value from virConnectGetLibVersion: {}",
                i
            ),
        }
    }

    pub fn hypervisor_version(&self) -> Result<Option<crate::version::Version>, VirtError> {
        let mut hv_ver: c_ulong = 0;
        match unsafe { virConnectGetVersion(self.0, &mut hv_ver) } {
            -1 => Err(VirtError::last_virt_error()),
            0 => Ok(if hv_ver == 0 {
                None
            } else {
                Some(crate::version::Version::from(hv_ver))
            }),
            i => panic!("Unexpected return value from virConnectGetVersion: {}", i),
        }
    }

    /// Closes the connection. If this connection has been cloned it just decrements the
    /// reference count. The connection is actually closed when the last instance is closed.
    pub fn close(self) -> Result<(), VirtError> {
        self.close_internal()
    }

    fn close_internal(&self) -> Result<(), VirtError> {
        match unsafe { virConnectClose(self.0) } {
            -1 => Err(VirtError::last_virt_error()),
            _ => Ok(()),
        }
    }
}

impl crate::Wrapper for Connection {
    type Ptr = virConnectPtr;

    /// Returns an instance backed by the given pointer.
    ///
    /// # Safety
    ///
    /// This method assumes it can take over ownership of the connection behind the pointer.
    /// The instance will call `virConnectClose` on the given pointer when it goes out of scope.
    unsafe fn from_ptr(ptr: virConnectPtr) -> Self {
        Self(ptr)
    }

    /// Returns a pointer to the underlying libvirt struct. The pointer is valid
    /// as long as this connection instance is in scope.
    fn as_ptr(&self) -> virConnectPtr {
        self.0
    }
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        assert_eq!(
            unsafe { virConnectRef(self.0) },
            0,
            "Unexpected error from virConnectRef"
        );
        Self(self.0)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if let Err(e) = self.close_internal() {
            log::error!("Error when closing connection: {}", e);
        }
    }
}
