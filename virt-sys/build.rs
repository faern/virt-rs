use std::{env, ffi::OsString, path::PathBuf};

/// Edit this to the name of the library you write bindings for. Without "lib" at the start.
const LIB_NAME: &str = "virt";

cfg_if::cfg_if! {
    if #[cfg(feature = "libvirt-5-7-0")] {
        const MIN_VERSION: &str = "5.7.0";
    } else if #[cfg(feature = "libvirt-5-6-0")] {
        const MIN_VERSION: &str = "5.6.0";
    } else if #[cfg(feature = "libvirt-5-5-0")] {
        const MIN_VERSION: &str = "5.5.0";
    } else if #[cfg(feature = "libvirt-5-4-0")] {
        const MIN_VERSION: &str = "5.4.0";
    } else if #[cfg(feature = "libvirt-5-3-0")] {
        const MIN_VERSION: &str = "5.3.0";
    } else if #[cfg(feature = "libvirt-5-2-0")] {
        const MIN_VERSION: &str = "5.2.0";
    } else if #[cfg(feature = "libvirt-5-1-0")] {
        const MIN_VERSION: &str = "5.1.0";
    } else {
        const MIN_VERSION: &str = "5.0.0";
    }
}

lazy_static::lazy_static! {
    static ref FULL_LIB_NAME: String = format!("lib{}", LIB_NAME);
    static ref LIB_DIR_VAR: String = format!("{}_LIB_DIR", FULL_LIB_NAME.to_uppercase());
    static ref STATIC_VAR: String = format!("{}_STATIC", FULL_LIB_NAME.to_uppercase());
}

fn main() {
    let static_linking = if link_statically() { "static=" } else { "" };

    // If the path to the library is specified in the environment variable, use that
    if let Some(lib_dir) = get_env(&LIB_DIR_VAR).map(PathBuf::from) {
        if !lib_dir.is_dir() {
            println!(
                "cargo:warning=Directory denoted by {} does not exist: {}",
                *LIB_DIR_VAR,
                lib_dir.display(),
            );
        }
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
        println!("cargo:rustc-link-lib={}{}", static_linking, LIB_NAME);
    } else {
        // Trying with pkg-config instead
        println!("Minimum {} version: {}", *FULL_LIB_NAME, MIN_VERSION);
        pkg_config::Config::new()
            .atleast_version(MIN_VERSION)
            .probe(&FULL_LIB_NAME)
            .expect("pkg_config can't find the library");
    }
}

fn get_env(var: &'static str) -> Option<OsString> {
    println!("cargo:rerun-if-env-changed={}", var);
    env::var_os(var)
}

fn get_env_str(var: &'static str) -> Option<String> {
    get_env(var).map(|s| {
        s.into_string()
            .expect(&format!("Variable {} is not correct UTF-8", var))
    })
}

/// Checks <LIBRARY_NAME>_STATIC for an explicit linking mode. Otherwise falls back on
/// target defaults.
fn link_statically() -> bool {
    match get_env_str(&STATIC_VAR).as_ref().map(String::as_str) {
        Some("0") => false,
        Some("1") => true,
        Some(_) => {
            println!(
                "cargo:warning=Variable {} set to unexpected value, use 0 or 1",
                *STATIC_VAR
            );
            link_statically_default()
        }
        None => link_statically_default(),
    }
}

/// The linking mode usually preferred for this target.
fn link_statically_default() -> bool {
    if get_env_str("CARGO_CFG_TARGET_ENV")
        .expect("Unknown target env")
        .as_str()
        == "musl"
    {
        true
    } else {
        match get_env_str("CARGO_CFG_TARGET_OS")
            .expect("Unknown OS")
            .as_str()
        {
            "macos" => true,
            "windows" => true,
            "linux" | "freebsd" | "dragonfly" | "openbsd" | "netbsd" => false,
            // Default to dynamic linking on all other platforms
            _ => false,
        }
    }
}
