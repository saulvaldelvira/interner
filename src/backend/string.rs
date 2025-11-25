use core::borrow::Borrow;

use crate::backend::Internable;
use crate::Backend;

struct Span {
    pub offset: usize,
    pub len: usize,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, PartialOrd, Ord)]
pub struct Symbol {
    pub offset: u32,
    pub len: u32,
}

impl Symbol {
    pub const fn from_usize(val: usize) -> Self {
        Self {
            offset: (val >> 32) as u32,
            len: (val & !( (!0) << 32 ) ) as u32,
        }
    }
    pub const fn as_usize(&self) -> usize {
        ((self.offset as usize) << 32) | self.len as usize
    }

    pub const fn is_inlined(&self) -> bool {
        self.len != u32::MAX
    }

    pub const fn new_inlined(offset: u32, len: u32) -> Self {
        Self { offset, len }
    }

    pub const fn new_indexed(index: usize) -> Self {
        Self { offset: index as u32, len: u32::MAX }
    }
}

/// Backend for strings
#[derive(Default)]
pub struct StringBackend {
    buf: String,
    spans: Vec<Span>,
}

impl Backend<str> for StringBackend {
    type Symbol = Symbol;

    fn get(&self, sym: Symbol) -> Option<&str> {
        let (offset, len) = if sym.is_inlined() {
            (sym.offset as usize, sym.len as usize)
        } else {
            let span = self.spans.get(sym.offset as usize)?;
            (span.offset, span.len)
        };
        let src = &self.buf[offset..offset + len];
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

        if len < u32::MAX as usize && offset <= u32::MAX as usize {
            Symbol::new_inlined(offset as u32, len as u32)
        } else {
            let span = Span { offset, len };
            let offset = b.spans.len() as u32;
            b.spans.push(span);
            Symbol { offset, len: u32::MAX }
        }
    }
}
