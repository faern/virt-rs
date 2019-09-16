use virt_sys::{
    virError, virErrorPtr, virFreeError, virSaveLastError, VIR_ERR_ERROR, VIR_ERR_NONE,
    VIR_ERR_WARNING,
};
use std::borrow::Cow;
use std::ffi::CStr;
use std::fmt;


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
        write!(f, "[{}] {}", level, self.message())
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
