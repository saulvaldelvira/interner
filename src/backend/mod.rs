mod string_buf;
use std::hash::Hash;

mod vec;
pub use vec::VecBackend;

pub use string_buf::StringBuf;

/// Allows to specify a default backend for some type
///
/// # Example
/// ```
/// use interns::DefaultBackend;
///
/// /* Get the default backend for the str type */
/// let backend = <str as DefaultBackend>::build_backend();
/// ```
pub trait DefaultBackend {
    type B: Backend<Self>;
    fn build_backend() -> Self::B;
}

impl<T: Sized + Clone> DefaultBackend for T {
    type B = VecBackend<T>;

    fn build_backend() -> Self::B {
        VecBackend::default()
    }
}

impl DefaultBackend for str {
    type B = StringBuf;

    fn build_backend() -> Self::B {
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
    fn get(&self, sym: Self::Symbol) -> Option<&T> ;
}
