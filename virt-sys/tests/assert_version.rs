use std::ptr;

#[test]
fn assert_linked_to_same_as_header() {
    let header_version = virt_sys::LIBVIR_VERSION_NUMBER as u64;

    let mut linked_version = 0;
    assert_eq!(
        unsafe { virt_sys::virGetVersion(&mut linked_version, ptr::null(), ptr::null_mut()) },
        0,
        "Unexpected return value from virGetVersion",
    );
    assert_ne!(linked_version, 0, "Unable to actually get libvirt version");
    assert_eq!(
        header_version, linked_version,
        "Linked to libvirt version is different than bindings are generated for"
    );
}
