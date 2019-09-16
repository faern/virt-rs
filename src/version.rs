use virt_sys::virGetVersion;
use std::os::raw::c_ulong;
use std::ptr;

use crate::error::VirtError;

/// A software version.
pub struct Version {
    /// The major version number.
    pub major: u64,
    /// The minor version number.
    pub minor: u64,
    /// The release version number.
    pub release: u64,
}

impl From<c_ulong> for Version {
    fn from(lib_ver: c_ulong) -> Self {
        Version {
            major: lib_ver / 1_000_000,
            minor: (lib_ver % 1_000_000) / 1_000,
            release: (lib_ver % 1_000),
        }
    }
}

/// Returns the version of the backing libvirt C implementation in use by this library.
pub fn lib_version() -> Result<Version, VirtError> {
    unparsed_lib_version().map(Version::from)
}

/// Returns the unparsed version of the backing libvirt C implementation in use by this library.
fn unparsed_lib_version() -> Result<c_ulong, VirtError> {
    let mut lib_ver: c_ulong = 0;
    match unsafe { virGetVersion(&mut lib_ver, ptr::null(), ptr::null_mut()) } {
        -1 => Err(VirtError::last_virt_error()),
        0 => Ok(lib_ver),
        i => panic!("Unexpected return value from virGetVersion: {}", i),
    }
}
