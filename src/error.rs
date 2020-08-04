use std::borrow::Cow;
use std::ffi::CStr;
use std::fmt;

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    VirtError(VirtError),
    InvalidUri(std::ffi::NulError),
    InvalidXml(std::ffi::NulError),
    InvalidName(std::ffi::NulError),
    Utf8Error(std::str::Utf8Error),
}

impl From<VirtError> for Error {
    fn from(e: VirtError) -> Self {
        Error::VirtError(e)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            VirtError(e) => e.fmt(f),
            InvalidUri(_) => "Invalid URI".fmt(f),
            InvalidXml(_) => "Invalid XML".fmt(f),
            InvalidName(_) => "Invalid name".fmt(f),
            Utf8Error(_) => "String is not valid UTF-8".fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use Error::*;
        match self {
            VirtError(e) => e.source(),
            InvalidUri(e) => Some(e),
            InvalidXml(e) => Some(e),
            InvalidName(e) => Some(e),
            Utf8Error(e) => Some(e),
        }
    }
}

/// An error from libvirt.
pub struct VirtError(virt_sys::virErrorPtr);

// Safety: libvirt is thread safe since 0.6.0. It can handle multiple threads making calls to the
// same virError instance.
unsafe impl Send for VirtError {}
unsafe impl Sync for VirtError {}

impl VirtError {
    /// Returns the last error that happened inside libvirt.
    ///
    /// Not really intended for direct use. The methods of this library
    /// will return one of these when they happen.
    ///
    /// # Panics
    ///
    /// Panics when unable to allocate heap memory for the error struct.
    pub fn last_virt_error() -> Self {
        let error_ptr = unsafe { virt_sys::virSaveLastError() };
        if error_ptr.is_null() {
            panic!("Unable to allocate memory for libvirt error");
        }
        Self(error_ptr)
    }

    /// Returns a Rust reference to the underlying libvirt error struct.
    /// Allows reading some individual field values without `unsafe {}`.
    pub fn as_ref(&self) -> &virt_sys::virError {
        unsafe { &*self.0 }
    }

    /// Returns the human-readable informative error message given by libvirt.
    pub fn message(&self) -> Cow<str> {
        unsafe { CStr::from_ptr(self.as_ref().message) }.to_string_lossy()
    }
}

impl crate::Wrapper for VirtError {
    type Ptr = virt_sys::virErrorPtr;

    unsafe fn from_ptr(ptr: Self::Ptr) -> Self {
        Self(ptr)
    }

    fn as_ptr(&self) -> Self::Ptr {
        self.0
    }
}

impl Drop for VirtError {
    fn drop(&mut self) {
        unsafe { virt_sys::virFreeError(self.0) }
    }
}

impl std::error::Error for VirtError {}

impl fmt::Display for VirtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let level = match self.as_ref().level {
            virt_sys::VIR_ERR_NONE => "No error",
            virt_sys::VIR_ERR_WARNING => "Warning",
            virt_sys::VIR_ERR_ERROR => "Error",
            _ => "Unknown error level",
        };
        write!(f, "{}: {}", level, self.message())
    }
}

impl fmt::Debug for VirtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("VirtError")
            .field("code", &self.as_ref().code)
            .field("domain", &self.as_ref().domain)
            .field("level", &self.as_ref().level)
            .field("message", &self.message())
            .finish()
    }
}
