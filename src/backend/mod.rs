mod string_buf;
use std::hash::Hash;

mod vec;
pub use vec::VecBackend;

pub use string_buf::StringBuf;

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

pub type DefaultBackend<T> = <T as DefaultBackendBuilder>::Backend;

impl<T: Sized + Clone> DefaultBackendBuilder for T {
    type Backend = VecBackend<T>;

    fn build_backend() -> Self::Backend {
        VecBackend::default()
    }
}

impl DefaultBackendBuilder for str {
    type Backend = StringBuf;

    fn build_backend() -> Self::Backend {
        StringBuf::default()
    }
}

pub trait BackendSymbol: Clone + Copy + Hash + Eq + PartialEq {}
impl<T> BackendSymbol for T where T: Clone + Copy + Hash + Eq + PartialEq {}

/// Backend for the [Interner](super::Interner)
pub trait Backend<T: ?Sized> {
    type Symbol: BackendSymbol;

    /// Intern the element
    fn intern(&mut self, src: &T) -> Self::Symbol;

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
