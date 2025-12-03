use core::borrow::Borrow;
use core::hash::BuildHasher;
use core::mem::MaybeUninit;

use hashbrown::hash_map::RawEntryMut;

use crate::backend::Internable;
use crate::{Backend, Interner, StringInterner};

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

impl StringBackend {
    fn prefill(&mut self, strings: &[(&str, Symbol)]) {
        assert!(self.spans.is_empty());
        for (string, expected_sym) in strings {
            let span = Span {
                offset: self.buf.len(),
                len: string.len(),
            };
            self.buf.push_str(string);
            let n = self.spans.len();
            self.spans.push(span);
            let sym = Symbol::new_indexed(n);
            assert_eq!(sym, *expected_sym);
        }
    }
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

/// A helper struct to build prefilled interners
///
/// This builder pre-fills the interner with a set of symbols defined at compile time.
///
/// # Example
/// ```
/// use interns::backend::string::*;
///
/// const BUILDER: StringInternerBuilder<4> = StringInternerBuilder::with_const_symbols([
///     "static", "const", "int", "fn"
/// ]);
/// const KWSTATIC: Symbol = BUILDER.symbol_at(0);
/// const KWCONST: Symbol = BUILDER.symbol_at(1);
/// const KWINT: Symbol = BUILDER.symbol_at(2);
/// const KWFN: Symbol = BUILDER.symbol_at(3);
///
/// let mut interner = BUILDER.build();
///
/// let sym_static = interner.get_or_intern("static");
/// assert_eq!(sym_static, KWSTATIC);
/// let sym_const = interner.get_or_intern("const");
/// assert_eq!(sym_const, KWCONST);
/// ```
pub struct StringInternerBuilder<const N: usize>([(&'static str, Symbol); N]);

impl<const N: usize> StringInternerBuilder<N> {
    /// Builds a [StringInternerBuilder] with the given const symbols
    pub const fn with_const_symbols(predefined: [&str; N]) -> Self {
        let mut result: [MaybeUninit<(&str, Symbol)>; N] = [ const { MaybeUninit::uninit() }; N ];
        let mut i = 0;
        while i < N {
            result[i] = MaybeUninit::new((predefined[i], Symbol::new_indexed(i)));
            i += 1;
        }
        Self(unsafe { core::mem::transmute_copy(&result) })
    }
    pub const fn symbol_at(&self, idx: usize) -> Symbol { self.0[idx].1 }
    pub const fn string_at(&self, idx: usize) -> &'static str { self.0[idx].0 }

    /// Builds a [StringInterner] with the pre-defined symbols given on [Self::with_const_symbols]
    pub fn build(&self) -> StringInterner {
        let mut i = StringInterner::new();
        i.prefill(&self.0);
        i
    }
}

impl<H: BuildHasher> Interner<str, StringBackend, H> {
    fn prefill(&mut self, syms: &[(&str, Symbol)]) {
        let Self { hasher, backend, .. } = self;
        backend.prefill(syms);
        for (string, sym) in syms {
            let hash = hasher.hash_one(string);
            let entry = self.set.raw_entry_mut().from_hash(hash, |s| s == sym);
            if let RawEntryMut::Vacant(vacant) = entry {
                vacant.insert_with_hasher(hash, *sym, (), |s| {
                    let s = unsafe { backend.get_unchecked(*s) };
                    hasher.hash_one(s)
                });
            }
        }
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
