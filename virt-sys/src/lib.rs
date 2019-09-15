#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

cfg_if::cfg_if! {
    if #[cfg(feature = "libvirt-5-7-0")] {
        mod libvirt_5_7_0;
        pub use self::libvirt_5_7_0::*;
    } else if #[cfg(feature = "libvirt-5-6-0")] {
        mod libvirt_5_6_0;
        pub use self::libvirt_5_6_0::*;
    } else if #[cfg(feature = "libvirt-5-5-0")] {
        mod libvirt_5_5_0;
        pub use self::libvirt_5_5_0::*;
    } else if #[cfg(feature = "libvirt-5-4-0")] {
        mod libvirt_5_4_0;
        pub use self::libvirt_5_4_0::*;
    } else if #[cfg(feature = "libvirt-5-3-0")] {
        mod libvirt_5_3_0;
        pub use self::libvirt_5_3_0::*;
    } else if #[cfg(feature = "libvirt-5-2-0")] {
        mod libvirt_5_2_0;
        pub use self::libvirt_5_2_0::*;
    } else if #[cfg(feature = "libvirt-5-1-0")] {
        mod libvirt_5_1_0;
        pub use self::libvirt_5_1_0::*;
    } else {
        mod libvirt_5_0_0;
        pub use self::libvirt_5_0_0::*;
    }
}
