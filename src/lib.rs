/// Re-export of the underlying FFI bindings to the C version of libvirt.
pub use virt_sys;

/// Types related to connecting to a hypervisor.
pub mod connection;
pub use connection::Connection;

mod error;
pub use error::{Error, VirtError};
