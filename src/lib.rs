use hashbrown::hash_map::RawEntryMut;
use hashbrown::{DefaultHashBuilder, HashMap};
use std::hash::{BuildHasher, Hash};

mod backend;
use backend::{Backend, DefaultBackend, StringBuf};

pub type Symbol<T, B = <T as DefaultBackend>::B> = <B as Backend<T>>::Symbol;

pub type StringInterner = Interner<str,StringBuf>;

/// Interner
///
/// This struct is responsible for tracking objects and
/// interning them.
///
/// # Example
/// ```
/// use interns::*;
///
/// let mut interner = Interner::<str>::default();
///
/// let a = interner.get_or_intern("hello");
/// let b = interner.get_or_intern("world");
/// let c = interner.get_or_intern("hello");
///
/// let a_resolv = interner.resolve(a);
/// let b_resolv = interner.resolve(b);
/// let c_resolv = interner.resolve(c);
///
/// assert_eq!(a_resolv, Some("hello"));
/// assert_eq!(b_resolv, Some("world"));
/// assert_eq!(c_resolv, Some("hello"));
///
/// assert_eq!(a, c);
/// assert_ne!(a, b);
/// assert_ne!(b, c);
/// ```
pub struct Interner<
    T,
    B = <T as DefaultBackend>::B,
    H = DefaultHashBuilder
>
where
    T: Hash + Eq + PartialEq + ?Sized,
    H: BuildHasher,
    B: Backend<T>,
{
    backend: B,
    set: HashMap<B::Symbol, (), ()>,
    hasher: H,
}

impl<T, B, H> Interner<T, B, H>
where
    T: Hash + Eq + PartialEq + ?Sized,
    H: BuildHasher,
    B: Backend<T>,
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

    pub fn get_or_intern(&mut self, src: &T) -> B::Symbol {
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

    pub fn resolve(&self, sym: B::Symbol) -> Option<&T> {
        self.backend.get(sym)
    }
}

impl<T,B> Default for Interner<T,B>
where
    T: Hash + Eq + PartialEq + ?Sized,
    B: Backend<T> + Default
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn string() {
        let mut interner = Interner::<str>::default();

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

    #[test]
    fn test_struct() {

        #[derive(Debug,Clone,Eq,PartialEq,Hash)]
        struct S {
            num: i32,
            s: String
        }

        let mut interner = Interner::<S>::default();

        let hello = S { num: 12, s: "hello".to_string() };
        let world = S { num: 13, s: "world".to_string() };

        let a = interner.get_or_intern(&hello);
        let b = interner.get_or_intern(&world);
        let c = interner.get_or_intern(&hello);

        assert_eq!(a, c);
        assert_ne!(a, b);
        assert_ne!(b, c);

        let hello_res = interner.resolve(a).unwrap();
        let world_res = interner.resolve(b).unwrap();
        let hello_res2 = interner.resolve(c).unwrap();

        assert_eq!(hello_res, &hello);
        assert_eq!(world_res, &world);
        assert_eq!(hello_res2, &hello);
    }
}
