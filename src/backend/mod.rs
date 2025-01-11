mod string_buf;
use std::hash::Hash;

pub use string_buf::StringBuf;

pub type DefaultBackend = StringBuf;

pub trait BackendSymbol: Clone + Copy + Hash + Eq + PartialEq {}
impl<T> BackendSymbol for T where T: Clone + Copy + Hash + Eq + PartialEq {}

pub trait Backend {
    type Symbol: BackendSymbol;

    fn intern(&mut self, src: &str) -> Self::Symbol;

    fn get(&self, sym: Self::Symbol) -> Option<&str>;

    unsafe fn get_unchecked(&self, sym: Self::Symbol) -> &str {
        unsafe { self.get(sym).unwrap_unchecked() }
    }
}
