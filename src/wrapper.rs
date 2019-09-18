/// Trait for types that wrap a C pointer and provide a safe interface for some underlying unsafe
/// C type.
pub trait Wrapper {
    type Ptr;

    /// Unsafely construct this wrapper type from a pointer to the corresponding C type.
    ///
    /// # Safety
    ///
    /// The wrapper type will usually assume full ownership of the underlying C type. It might
    /// perform any action on the type in this method or after it has returned. Usually the wrapper
    /// free/close the underlying type when they go out of scope.
    unsafe fn from_ptr(ptr: Self::Ptr) -> Self;

    /// Returns the pointer to the underlying C type. Can be used to perform `virt_sys`
    /// operations directly on the type, in case the safe API layer does not yet expose a certain
    /// functionality.
    fn as_ptr(&self) -> Self::Ptr;
}
