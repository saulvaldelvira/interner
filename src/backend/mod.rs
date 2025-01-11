mod string_buf;
use std::hash::Hash;

mod vec;
pub use vec::VecBackend;

pub use string_buf::StringBuf;

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

pub trait Backend<T: ?Sized> {
    type Symbol: BackendSymbol;

    fn intern(&mut self, src: &T) -> Self::Symbol;

    fn get(&self, sym: Self::Symbol) -> Option<&T> ;

    unsafe fn get_unchecked(&self, sym: Self::Symbol) -> &T {
        unsafe { self.get(sym).unwrap_unchecked() }
    }
}
