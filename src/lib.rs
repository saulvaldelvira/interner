use hashbrown::hash_map::RawEntryMut;
use hashbrown::{DefaultHashBuilder, HashMap};
use std::hash::BuildHasher;

mod backend;
use backend::{Backend, DefaultBackend};

pub struct Interner<B = DefaultBackend, H = DefaultHashBuilder>
where
    H: BuildHasher,
    B: Backend,
{
    backend: B,
    set: HashMap<B::Symbol, (), ()>,
    hasher: H,
}

impl<B, H> Interner<B, H>
where
    H: BuildHasher,
    B: Backend,
{
    pub fn new() -> Self
    where
        B: Default,
        H: Default,
    {
        Self {
            backend: B::default(),
            set: HashMap::default(),
            hasher: H::default(),
        }
    }

    pub fn with_hasher(hasher: H) -> Self
    where
        B: Default,
    {
        Self {
            backend: B::default(),
            set: HashMap::default(),
            hasher,
        }
    }

    pub fn with_backend(backend: B) -> Self
    where
        H: Default,
    {
        Self {
            backend,
            set: HashMap::default(),
            hasher: H::default(),
        }
    }

    pub fn with_backend_and_hasher(backend: B, hasher: H) -> Self {
        Self {
            backend,
            hasher,
            set: HashMap::default(),
        }
    }

    pub fn get_or_intern(&mut self, src: &str) -> B::Symbol {
        let Self {
            backend,
            set,
            hasher,
        } = self;

        let hash = hasher.hash_one(src);

        let entry = set
            .raw_entry_mut()
            .from_hash(hash, |&sym| src == unsafe { backend.get_unchecked(sym) });

        let k = match entry {
            RawEntryMut::Occupied(occupied) => occupied.into_key(),
            RawEntryMut::Vacant(vacant) => {
                let sym = backend.intern(src);
                vacant
                    .insert_with_hasher(hash, sym, (), |sym| {
                        let src = unsafe { backend.get_unchecked(*sym) };
                        hasher.hash_one(src)
                    })
                    .0
            }
        };

        *k
    }

    pub fn resolve(&self, sym: B::Symbol) -> Option<&str> {
        self.backend.get(sym)
    }
}

impl Default for Interner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut interner = Interner::default();

        let a = interner.get_or_intern("hello");
        let b = interner.get_or_intern("world");
        let c = interner.get_or_intern("hello");

        let mut map = HashMap::new();
        map.insert(a, "hello");
        map.insert(b, "world");

        assert_eq!(a, c);
        assert_ne!(a, b);
        assert_ne!(b, c);

        let hello = *map.get(&c).unwrap();
        assert_eq!(hello, "hello");
    }
}
