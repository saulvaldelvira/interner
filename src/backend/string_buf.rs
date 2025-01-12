use crate::Backend;

struct Span {
    pub offset: usize,
    pub len: usize,
}

/// Backend for strings
#[derive(Default)]
pub struct StringBuf {
    buf: String,
    spans: Vec<Span>,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
#[repr(transparent)]
pub struct Symbol(usize);

impl Backend<str> for StringBuf {
    type Symbol = Symbol;

    fn intern(&mut self, src: &str) -> Self::Symbol {
        let offset = self.buf.len();
        let len = src.len();
        self.buf.push_str(src);

        let span = Span { offset, len };
        let sym = Symbol(self.spans.len());
        self.spans.push(span);
        sym
    }

    fn get(&self, sym: Symbol) -> Option<&str> {
        let span = self.spans.get(sym.0)?;
        let src = &self.buf[span.offset..span.offset + span.len];
        Some(src)
    }
}
