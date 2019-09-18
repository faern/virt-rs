use std::borrow::Cow;
use std::ffi::CStr;
use std::fmt;
use virt_sys::{
    virError, virErrorPtr, virFreeError, virSaveLastError, VIR_ERR_ERROR, VIR_ERR_NONE,
    VIR_ERR_WARNING,
};

#[derive(err_derive::Error, Debug)]
pub enum Error {
    #[error(display = "Error inside libvirt")]
    VirtError(#[error(cause)] VirtError),
    #[error(display = "Invalid URI")]
    InvalidUri(#[error(cause)] std::ffi::NulError),
    #[error(display = "String is not valid UTF-8")]
    Utf8Error(#[error(cause)] std::str::Utf8Error),

    /// You should never match on this error variant. This enum can grow, and the breaking version
    /// number won't be bumped.
    #[doc(hidden)]
    #[error(display = "__Nonexhaustive")]
    __Nonexhaustive,
}

impl From<VirtError> for Error {
    fn from(e: VirtError) -> Self {
        Error::VirtError(e)
    }
}

/// An error from libvirt.
pub struct VirtError(virErrorPtr);

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
        let error_ptr = unsafe { virSaveLastError() };
        if error_ptr.is_null() {
            panic!("Unable to allocate memory for libvirt error");
        }
        Self(error_ptr)
    }

    /// Returns a `VirtError` object using the provided pointer.
    ///
    /// # Safety
    ///
    /// This new error instance will assume ownership of the virError behind the pointer.
    /// The returned instance will call `virFreeError(ptr)` on the given pointer when dropped.
    pub unsafe fn from_ptr(ptr: virErrorPtr) -> Self {
        Self(ptr)
    }

    /// Returns a pointer to the underlying libvirt error. Pointer is valid
    /// as long as this `Error` is in scope.
    pub fn as_ptr(&self) -> virErrorPtr {
        self.0
    }

    /// Returns a Rust reference to the underlying libvirt error struct.
    /// Allows reading some individual field values without `unsafe {}`.
    pub fn as_ref(&self) -> &virError {
        unsafe { &*self.0 }
    }

    /// Returns the human-readable informative error message given by libvirt.
    pub fn message(&self) -> Cow<str> {
        unsafe { CStr::from_ptr(self.as_ref().message) }.to_string_lossy()
    }
}

impl Drop for VirtError {
    fn drop(&mut self) {
        unsafe { virFreeError(self.0) }
    }
}

impl std::error::Error for VirtError {}

impl fmt::Display for VirtError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let level = match self.as_ref().level {
            VIR_ERR_NONE => "No error",
            VIR_ERR_WARNING => "Warning",
            VIR_ERR_ERROR => "Error",
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
