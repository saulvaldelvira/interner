//! Storage for the interned elements

use core::borrow::Borrow;
use std::hash::Hash;

mod string;
pub use string::StringBackend;

mod vec;
pub use vec::VecBackend;


/// Allows to specify a default backend for some type
///
/// # Example
/// ```
/// use interns::DefaultBackendBuilder;
///
/// /* Get the default backend for the str type */
/// let backend = <str as DefaultBackendBuilder>::build_backend();
/// ```
pub trait DefaultBackendBuilder {
    type Backend: Backend<Self>;
    fn build_backend() -> Self::Backend;
}

/// Resolves the default backend of a type
pub type DefaultBackend<T> = <T as DefaultBackendBuilder>::Backend;

impl<T: Sized + Clone> DefaultBackendBuilder for T {
    type Backend = VecBackend<T>;

    fn build_backend() -> Self::Backend {
        VecBackend::default()
    }
}

impl DefaultBackendBuilder for str {
    type Backend = StringBackend;

    fn build_backend() -> Self::Backend {
        StringBackend::default()
    }
}

impl<T> DefaultBackendBuilder for [T] {
    type Backend = VecBackend<T>;

    fn build_backend() -> Self::Backend {
        VecBackend::default()

    }
}

/// All the constraints for a [Symbol](Backend::Symbol)
pub trait BackendSymbol: Clone + Copy + Hash + Eq + PartialEq {}
impl<T> BackendSymbol for T where T: Clone + Copy + Hash + Eq + PartialEq {}

/// Backend for the [Interner](super::Interner)
pub trait Backend<T: ?Sized> {
    type Symbol: BackendSymbol;

    /// Intern an element into `self`
    fn intern<B>(&mut self, src: &B) -> Self::Symbol
    where
        T: Borrow<B>,
        B: Internable<T, Self> + ?Sized,
    {
        src.intern_into(self)
    }

    /// Resolve the symbol
    fn get(&self, sym: Self::Symbol) -> Option<&T>;

    /// Resolves the symbol, without checking if it exists on
    /// the backend.
    ///
    /// # Safety
    /// The caller must ensure that the symbol was retreived
    /// from a call to this backend's [intern](Backend::intern) function.
    unsafe fn get_unchecked(&self, sym: Self::Symbol) -> &T {
        let val = self.get(sym);
        debug_assert!(val.is_some());
        /* SAFETY: the caller ensures that the symbol is valid for `self` */
        unsafe { val.unwrap_unchecked() }
    }
}

/// Defines how to intern a type into a [Backend]
///
/// This trait is needed because some backends have different
/// constraints about what types they can intern
///
/// For example: The [StringBackend] can intern any kind of
/// type that implements [`AsRef<str>`].
/// But we can't just hack an `AsRef<T>` bound for any `Backend<T>`,
/// since that would make it impossible for the [VecBackend] to receive
/// a reference of `T` and clone it. (T doesn't implement `AsRef<T>`).
pub trait Internable<T, B>
where
    T: Borrow<Self> + ?Sized,
    B: Backend<T> + ?Sized,
{
    fn intern_into(&self, b: &mut B) -> B::Symbol;
}

