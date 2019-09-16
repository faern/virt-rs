macro_rules! cvt_null {
    ($f:expr) => {{
        let ret = $f;
        if ret.is_null() {
            Err(crate::VirtError::last_virt_error())
        } else {
            Ok(ret)
        }
    }};
}
