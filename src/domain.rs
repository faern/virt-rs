use crate::{wrapper::Wrapper, Connection, Error, VirtError};
use std::{ffi::CString, mem};

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
bitflags::bitflags! {
    pub struct DeviceModifyFlags: u32 {
        const CONFIG = virt_sys::VIR_DOMAIN_DEVICE_MODIFY_CONFIG;
        const CURRENT = virt_sys::VIR_DOMAIN_DEVICE_MODIFY_CURRENT;
        const LIVE = virt_sys::VIR_DOMAIN_DEVICE_MODIFY_LIVE;
        const FORCE = virt_sys::VIR_DOMAIN_DEVICE_MODIFY_FORCE;
    }
}

/// Various ways to handle the termination of a domain when calling [Domain::destroy].
pub enum DestroyMode {
    /// Never forcefully kill the domain. If it does not shut down gracefully in a timely manner,
    /// an error is returned.
    Graceful,

    /// Kills the domain (e.g. SIGKILL) if it does not shut down gracefully a while after receiving
    /// the first shutdown signal.
    ///
    /// Killing a domain may produce undesirable results, for example unflushed disk cache in the
    /// guest.
    KillAfterTimeout,
}

pub struct Domain(virt_sys::virDomainPtr);

// Safety: libvirt is thread safe since 0.6.0. It can handle multiple threads making calls to the
// same virConnect instance.
unsafe impl Send for Domain {}

impl Domain {
    /// See [Connection::create_domain].
    pub(crate) fn create_from_xml(
        connection: &Connection,
        xml: &str,
        flags: CreateFlags,
    ) -> Result<Self, Error> {
        let xml_cstr = CString::new(xml).map_err(Error::InvalidXml)?;
        let ptr = cvt_null!(unsafe {
            virt_sys::virDomainCreateXML(connection.as_ptr(), xml_cstr.as_ptr(), flags.bits())
        })?;
        Ok(Domain(ptr))
    }

    /// Attach a virtual device to a domain, using the flags parameter to control how the device is
    /// attached. [DeviceModifyFlags::CURRENT] specifies that the device allocation is made based
    /// on current domain state. [DeviceModifyFlags::LIVE] specifies that the device shall be
    /// allocated to the active domain instance only and is not added to the persisted domain
    /// configuration. [DeviceModifyflags::CONFIG] specifies that the device shall be allocated to
    /// the persisted domain configuration only. Note that the target hypervisor must return an
    /// error if unable to satisfy flags. E.g. the hypervisor driver will return failure if LIVE
    /// is specified but it only supports modifying the persisted device allocation.
    ///
    /// Be aware that hotplug changes might not persist across a domain going into S4 state
    /// (also known as hibernation) unless you also modify the persistent domain definition.
    pub fn attach_device(&self, xml: &str, flags: DeviceModifyFlags) -> Result<(), Error> {
        let xml_cstr = CString::new(xml).map_err(Error::InvalidXml)?;
        match unsafe {
            virt_sys::virDomainAttachDeviceFlags(self.0, xml_cstr.as_ptr(), flags.bits())
        } {
            -1 => Err(Error::from(VirtError::last_virt_error())),
            _ => Ok(()),
        }
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

    /// Resume a suspended domain, the process is restarted from the state where it was frozen by
    /// calling [Domain::suspend]. This function may require privileged access. Moreover, resume may not
    /// be supported if domain is in some special state like VIR_DOMAIN_PMSUSPENDED.
    pub fn resume(&self) -> Result<(), VirtError> {
        match unsafe { virt_sys::virDomainResume(self.0) } {
            -1 => Err(VirtError::last_virt_error()),
            _ => Ok(()),
        }
    }

    /// Suspends an active domain, the process is frozen without further access to CPU resources
    /// and I/O but the memory used by the domain at the hypervisor level will stay allocated. Use
    /// [Domain::resume] to reactivate the domain. This function may require privileged access.
    /// Moreover, suspend may not be supported if domain is in some special state like
    /// VIR_DOMAIN_PMSUSPENDED.
    pub fn suspend(&self) -> Result<(), VirtError> {
        match unsafe { virt_sys::virDomainSuspend(self.0) } {
            -1 => Err(VirtError::last_virt_error()),
            _ => Ok(()),
        }
    }

    /// Shutdown a domain, the domain object is still usable thereafter, but the domain OS is being
    /// stopped. Note that the guest OS may ignore the request. Additionally, the hypervisor may
    /// check and support the domain 'on_poweroff' XML setting resulting in a domain that reboots
    /// instead of shutting down. For guests that react to a shutdown request, the differences from
    /// [Domain::destroy] are that the guests disk storage will be in a stable state rather than
    /// having the (virtual) power cord pulled, and this command returns as soon as the shutdown
    /// request is issued rather than blocking until the guest is no longer running.
    pub fn shutdown(&self) -> Result<(), VirtError> {
        match unsafe { virt_sys::virDomainShutdown(self.0) } {
            -1 => Err(VirtError::last_virt_error()),
            _ => Ok(()),
        }
    }

    /// Destroy the domain object. The running instance is shutdown if not down already and all
    /// resources used by it are given back to the hypervisor. This function may require privileged
    /// access.
    ///
    /// Calling this method first requests that the guest terminate (e.g. SIGTERM), then waits
    /// for it to comply. After a reasonable timeout, if the guest still exists, then it depends
    /// on the `mode`, see [DestroyMode].
    pub fn destroy(&self, mode: DestroyMode) -> Result<(), VirtError> {
        let flags = match mode {
            DestroyMode::KillAfterTimeout => virt_sys::VIR_DOMAIN_DESTROY_DEFAULT,
            DestroyMode::Graceful => virt_sys::VIR_DOMAIN_DESTROY_GRACEFUL,
        };
        match unsafe { virt_sys::virDomainDestroyFlags(self.0, flags) } {
            -1 => Err(VirtError::last_virt_error()),
            _ => Ok(()),
        }
    }

    /// Free the domain object. The running instance is kept alive.
    /// If this is not explicitly called it will be called by the `Drop` implementation. And any
    /// error will be logged to the error level.
    ///
    /// The only reason to call this explicitly is if you want to handle the error in some other
    /// way than just logging it.
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
    type Ptr = virt_sys::virDomainPtr;

    unsafe fn from_ptr(ptr: Self::Ptr) -> Self {
        Self(ptr)
    }

    fn as_ptr(&self) -> Self::Ptr {
        self.0
    }
}

impl Clone for Domain {
    fn clone(&self) -> Self {
        let ret = unsafe { virt_sys::virDomainRef(self.0) };
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
