use crate::{Error, VirtError};
use std::ffi::{CStr, CString};
use std::os::raw::c_ulong;
use std::ptr;
use virt_sys::{
    virConnectClose, virConnectGetHostname, virConnectGetLibVersion, virConnectGetType,
    virConnectGetURI, virConnectGetVersion, virConnectPtr, virConnectRef,
};

/// A [Connection] builder.
#[derive(Debug, Default)]
pub struct Builder {
    uri: Option<CString>,
    flags: virt_sys::virConnectFlags,
}

impl Builder {
    /// Sets the URI to the hypervisor to connect to. If no URI is set the environment variable
    /// `LIBVIRT_DEFAULT_URI` will be used, if set. Otherwise if the client configuration file
    /// has the "uri_default" parameter set, then it will be used. Finally probing will be done
    /// to determine a suitable default driver to activate. This involves trying each hypervisor
    /// in turn until one successfully opens.
    ///
    /// URIs are documented at https://libvirt.org/uri.html
    pub fn uri(&mut self, uri: &str) -> Result<&mut Self, Error> {
        self.uri = Some(CString::new(uri).map_err(Error::InvalidUri)?);
        Ok(self)
    }

    /// Lower level version of [Builder::uri]. This version does not risk running into the
    /// `InvalidUri` error.
    pub fn uri_cstr(&mut self, uri: impl Into<CString>) -> &mut Self {
        self.uri = Some(uri.into());
        self
    }

    /// Sets whether or not the opened connection should be read only or not
    pub fn read_only(&mut self, read_only: bool) -> &mut Self {
        if read_only {
            self.flags |= virt_sys::VIR_CONNECT_RO;
        } else {
            self.flags &= !virt_sys::VIR_CONNECT_RO;
        }
        self
    }

    /// Sets whether or not the opened connection should try to resolve URI aliases or not.
    pub fn no_aliases(&mut self, no_aliases: bool) -> &mut Self {
        if no_aliases {
            self.flags |= virt_sys::VIR_CONNECT_NO_ALIASES;
        } else {
            self.flags &= !virt_sys::VIR_CONNECT_NO_ALIASES;
        }
        self
    }

    /// Tries to open a connection to the configured hypervisor.
    pub fn open(&self) -> Result<Connection, VirtError> {
        let uri_ptr = match &self.uri {
            Some(uri) => uri.as_ptr(),
            None => ptr::null(),
        };
        let connection_ptr = cvt_null!(unsafe {
            virt_sys::virConnectOpenAuth(uri_ptr, ptr::null_mut(), self.flags)
        })?;
        Ok(Connection(connection_ptr))
    }
}

pub struct Connection(virConnectPtr);

impl Connection {
    /// Returns a builder for [Connection]s.
    pub fn builder() -> Builder {
        Builder::default()
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
