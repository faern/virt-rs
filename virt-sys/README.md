# Bindings for libvirt

Low level FFI bindings to [`libvirt`].

For the high level, safe and Rust idiomatic bindings, see the `virt` crate in this repo.

[`libvirt`]: https://libvirt.org/


## libvirt version

Without any activated cargo features this will compile to libvirt-5.0.0 bindings.
Activate the corresponding version feature to get the newer API.
As long as libvirt follow semver the minor bumps should be API compatible.

An example showing how to depend on this library and get the version 6.1.0 API.
```toml
[dependencies]
virt = { git = "https://github.com/faern/virt-rs.git", features = ["libvirt-6-1-0"] }
```

## Dependencies and building

This library links towards the libvirt C library. So the development version of that must be
installed on the build system.

Fedora:
```bash
dnf install libvirt-devel
```
