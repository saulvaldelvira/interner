use core::borrow::Borrow;

use crate::backend::Internable;
use crate::Backend;

struct Span {
    pub offset: usize,
    pub len: usize,
}

/// Backend for strings
#[derive(Default)]
pub struct StringBackend {
    buf: String,
    spans: Vec<Span>,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[repr(transparent)]
pub struct Symbol(usize);

impl Symbol {
    pub fn as_usize(&self) -> usize { self.0 }
}

impl Backend<str> for StringBackend {
    type Symbol = Symbol;

    fn get(&self, sym: Symbol) -> Option<&str> {
        let span = self.spans.get(sym.0)?;
        let src = &self.buf[span.offset..span.offset + span.len];
        Some(src)
    }
}

impl<T> Internable<str, StringBackend> for T
where
    str: Borrow<T>,
    T: AsRef<str> + ?Sized
{
    fn intern_into(&self, b: &mut StringBackend) -> Symbol {
        let offset = b.buf.len();
        let src = self.as_ref();
        let len = src.len();
        b.buf.push_str(src);

        let span = Span { offset, len };
        let sym = Symbol(b.spans.len());
        b.spans.push(span);
        sym
    }
}
