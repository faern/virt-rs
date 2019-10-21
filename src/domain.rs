use crate::{wrapper::Wrapper, Connection, Error, VirtError};
use std::{ffi::CString, mem};
use virt_sys::{virDomainAttachDevice, virDomainCreateXML, virDomainPtr, virDomainRef};

bitflags::bitflags! {
    /// Flags affecting the starting of transient domains
    pub struct CreateFlags: u32 {
        ///  If the PAUSED flag is set, the guest domain will be started, but its CPUs will remain
        /// paused. The CPUs can later be manually started using the resume method.
        const PAUSED = virt_sys::VIR_DOMAIN_START_PAUSED;
        /// If the AUTODESTROY flag is set, the guest domain will be automatically destroyed when
        /// the [Connection] object is dropped. This will also happen if the client application
        /// crashes or loses its connection to the libvirtd daemon. Any domains marked for auto
        /// destroy will block attempts at migration, save-to-file, or snapshots.
        const AUTODESTROY = virt_sys::VIR_DOMAIN_START_AUTODESTROY;
        /// Avoid file system cache pollution
        const BYPASS_CACHE = virt_sys::VIR_DOMAIN_START_BYPASS_CACHE;
        /// Boot, discarding any managed save
        const FORCE_BOOT = virt_sys::VIR_DOMAIN_START_FORCE_BOOT;
        /// Validate the XML document against schema
        const VALIDATE =  virt_sys::VIR_DOMAIN_START_VALIDATE;
    }
}

pub struct Domain(virDomainPtr);

impl Domain {
    pub(crate) fn create_from_xml(
        connection: &Connection,
        xml: &str,
        flags: CreateFlags,
    ) -> Result<Self, Error> {
        let xml_cstr = CString::new(xml).map_err(Error::InvalidXml)?;
        let ptr = cvt_null!(unsafe {
            virDomainCreateXML(connection.as_ptr(), xml_cstr.as_ptr(), flags.bits())
        })?;
        Ok(Domain(ptr))
    }

    /// Rename a domain. Depending on each driver implementation it may be required that domain is
    /// in a specific state.
    ///
    /// There might be some attributes and/or elements in domain XML that if no value provided at
    /// XML defining time, libvirt will derive their value from the domain name. These are not
    /// updated by this API. Users are strongly advised to change these after the rename was
    /// successful.
    pub fn rename(&self, new_name: &str) -> Result<(), Error> {
        let new_name_cstr = CString::new(new_name).map_err(Error::InvalidName)?;
        match unsafe { virt_sys::virDomainRename(self.0, new_name_cstr.as_ptr(), 0) } {
            -1 => Err(Error::from(VirtError::last_virt_error())),
            _ => Ok(()),
        }
    }

    /// Free the domain object. The running instance is kept alive.
    /// If this is not explicitly called it will be called by the `Drop` implementation.
    pub fn free(self) -> Result<(), VirtError> {
        let result = self.free_internal();
        mem::forget(self);
        result
    }

    fn free_internal(&self) -> Result<(), VirtError> {
        match unsafe { virt_sys::virDomainFree(self.0) } {
            -1 => Err(VirtError::last_virt_error()),
            _ => Ok(()),
        }
    }
}

impl crate::Wrapper for Domain {
    type Ptr = virDomainPtr;

    unsafe fn from_ptr(ptr: Self::Ptr) -> Self {
        Self(ptr)
    }

    fn as_ptr(&self) -> Self::Ptr {
        self.0
    }
}

impl Clone for Domain {
    fn clone(&self) -> Self {
        let ret = unsafe { virDomainRef(self.0) };
        assert_eq!(ret, 0, "Unexpected error from virDomainRef");
        Self(self.0)
    }
}

impl Drop for Domain {
    fn drop(&mut self) {
        if let Err(e) = self.free_internal() {
            log::error!("Error when freeing domain: {}", e);
        }
    }
}
