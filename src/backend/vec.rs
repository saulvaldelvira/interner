use super::Backend;

/// Backend that stores elements inside a [Vec]
pub struct VecBackend<T> {
    buf: Vec<T>,
}

impl<T> Default for VecBackend<T> {
    fn default() -> Self {
        Self { buf: Default::default() }
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[repr(transparent)]
pub struct Symbol(usize);

impl<T: Clone> Backend<T> for VecBackend<T> {
    type Symbol = Symbol;

    fn intern(&mut self, src: &T) -> Self::Symbol {
        let sym = Symbol(self.buf.len());
        self.buf.push(src.clone());
        sym
    }

    fn get(&self, sym: Symbol) -> Option<&T> {
        let val = self.buf.get(sym.0)?;
        Some(val)
    }
}
