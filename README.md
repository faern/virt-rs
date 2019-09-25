# Bindings for libvirt

Safe and idiomatic Rust bindings to [`libvirt`].

For the automatically generated low level FFI bindings, see the `virt-sys` crate in this repo.

[`libvirt`]: https://libvirt.org/

## Prior/related work

There are existing bindings to libvirt available. Libvirt hosts what seems to be the official
bindings at https://libvirt.org/git/?p=libvirt-rust.git;a=summary. I tried to use those bingings
at first, but ran into too many obstacles. I then attempted to contribute some patches, but using
a mail list for this is just not going to cut it for me. There is no way I will expose SMTP to
my dev machine, and I did not manage to send a proper thread from my email client.

The issues I had with the official bindings:

* Explicitly states that they are a direct mapping of the underlying C API. Meaning it
  intentionally won't do proper resource freeing on dropped instances and other things one
  would expect from a nice Rust API.
* Unsoundness/invalid memory access in `Stream::recv`.
  https://www.redhat.com/archives/libvir-list/2019-September/msg00764.html
* Converts between C stings and Rust strings without any error handling. Panics on any string
  content that is not compatible in the other format.
* Unsoundness in all constructors. All safe wrappers can be created without `unsafe` with raw
  pointers. Meaning any subsequent use will cause undefined behavior:
  ```rust
  // This will segfault
  virt::connect::Connect::new(ptr::null_mut()).get_hostname()
  ```
  These constructons must be `unsafe`. They should probably be named `from_ptr` and be put further
  down in the `impl` block as well. They are not supposed to be the main way of creating the safe
  wrappers. https://www.redhat.com/archives/libvir-list/2019-September/msg01139.html
* No separation between raw FFI bindings and higher level safe bindings. The more standard design
  would be to have a separate `virt-sys` crate with `bindgen` generated direct bindings. This will
  make maintaining and verifying the raw bindings easier, and the source files of the higer level
  crate don't get cluttered with thousands of lines of `extern` declarations. It also directly
  exposes the entire surface area of libvirt. So even if ideal usage of the bindings never need
  to go down to this layer, it is possible, if the safe wrappers are missing some feature or it's
  implemented in a way that is not suitable for the current use case.
  Even if my bindings and the official ones were to remain separate projects forever, it would be
  beneficial for everyone if we shared the same `-sys` crate with low level bindings.
* Very nitpicky, but the code style is very C-isch instead of Rust. Explicit `return` statements
  where implicitly returning would work fine etc.

The best would of course be if there were one set of awesome bindings. So if you work on the
official bindings and would like to collaborate, let's do that!