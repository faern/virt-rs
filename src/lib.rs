/// Re-export of the underlying FFI bindings to the C version of libvirt.
pub use virt_sys;

mod error;
pub use error::VirtError;
