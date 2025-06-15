use core::borrow::Borrow;

use crate::backend::Internable;

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

impl<T> Backend<T> for VecBackend<T> {
    type Symbol = Symbol;

    fn get(&self, sym: Self::Symbol) -> Option<&T> {
        let val = self.buf.get(sym.0)?;
        Some(val)
    }
}

impl<T, Ref, Inter> Internable<T, VecBackend<T>> for Ref
where
    T: Borrow<Ref>,
    Ref: ToOwned<Owned = Inter> + ?Sized,
    Inter: Into<T>
{
    fn intern_into(&self, b: &mut VecBackend<T>) -> Symbol {
        let sym = Symbol(b.buf.len());
        b.buf.push(self.to_owned().into());
        sym
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct Span {
    start: usize,
    len: usize,
}

impl<T> Backend<[T]> for VecBackend<T> {
    type Symbol = Span;

    fn get(&self, sym: Self::Symbol) -> Option<&[T]> {
        let val = self.buf.get(sym.start..sym.start + sym.len)?;
        Some(val)
    }
}

impl<T: Clone> Internable<[T], VecBackend<T>> for [T] {
    fn intern_into(&self, b: &mut VecBackend<T>) -> Span {
        let start = b.buf.len();
        b.buf.extend_from_slice(self);
        let len = b.buf.len() - start;
        Span { start, len }
    }
}
