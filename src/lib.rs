//! An object interner library
//!
//! The main element of this crate is the [`Interner`] struct.
//!
//! It allows to build a "storage" of any kind of object that
//! avoids repetition and memory waste.
//!
//! # Example (String interner)
//! ```
//! use interns::*;
//!
//! let mut interner = Interner::<str>::default();
//!
//! let a = interner.get_or_intern("hello");
//! let b = interner.get_or_intern("world");
//! let c = interner.get_or_intern("hello");
//!
//! let a_resolv = interner.resolve(a);
//! let b_resolv = interner.resolve(b);
//! let c_resolv = interner.resolve(c);
//!
//! assert_eq!(a_resolv, Some("hello"));
//! assert_eq!(b_resolv, Some("world"));
//! assert_eq!(c_resolv, Some("hello"));
//!
//! assert_eq!(a, c);
//! assert_ne!(a, b);
//! assert_ne!(b, c);
//! ```

#![cfg_attr(not(feature = "hashbrown"), feature(hash_raw_entry))]

#[cfg(feature = "hashbrown")]
use hashbrown::{HashMap, hash_map::RawEntryMut};

#[cfg(not(feature = "hashbrown"))]
use std::collections::{HashMap, hash_map::RawEntryMut};

use std::hash::{BuildHasher, Hash, Hasher, RandomState};

pub mod backend;
pub use backend::{Backend, DefaultBackendBuilder, StringBuf};

pub type Symbol<T, B = <T as DefaultBackendBuilder>::Backend> = <B as Backend<T>>::Symbol;

pub type StringInterner = Interner<str,StringBuf>;

/// Dummy state that implements [Hasher] and [BuildHasher]
///
/// NOTE: This struct shouldn't be used.
/// All the methods it implements from the [Hasher] and [BuildHasher]
/// traits are marked as [unreachable].
///
/// It's only there to satisfy the HashMap's requirement that S: BuildHasher
#[derive(Default)]
pub struct DummyState;

impl Hasher for DummyState {
    fn finish(&self) -> u64 {
        unreachable!()
    }

    fn write(&mut self, _bytes: &[u8]) {
        unreachable!()
    }
}

impl BuildHasher for DummyState {
    type Hasher = DummyState;

    fn build_hasher(&self) -> Self::Hasher {
        unreachable!()
    }
}

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
    B = <T as DefaultBackendBuilder>::Backend,
    H = RandomState
>
where
    T: Hash + Eq + PartialEq + ?Sized,
    H: BuildHasher,
    B: Backend<T>,
{
    backend: B,
    set: HashMap<B::Symbol, (), DummyState>,
    hasher: H,
}

impl<T, B, H> Interner<T, B, H>
where
    T: Hash + Eq + PartialEq + ?Sized,
    H: BuildHasher,
    B: Backend<T>,
{
    /// Create a new Interner with a default [backend](Backend)
    /// and [hasher](BuildHasher)
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

    /// Create a new Interner with a default [backend](Backend) and
    /// the given [hasher](BuildHasher)
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

    /// Create a new Interner with a default [hasher](BuildHasher) and
    /// the given [backend](Backend)
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

    /// Create a new Interner with the given [backend](Backend)
    /// and [hasher](BuildHasher)
    pub fn with_backend_and_hasher(backend: B, hasher: H) -> Self {
        Self {
            backend,
            hasher,
            set: HashMap::default(),
        }
    }

    /// Turns a reference of T into the backend's [symbol](Backend::Symbol)
    pub fn get_or_intern(&mut self, src: &T) -> B::Symbol {
        let Self {
            backend,
            set,
            hasher,
        } = self;

        let hash = hasher.hash_one(src);

        let entry = set
            .raw_entry_mut()
            .from_hash(hash, |&sym| src == unsafe { backend.get(sym).unwrap_unchecked() });

        let k = match entry {
            RawEntryMut::Occupied(occupied) => occupied.into_key(),
            RawEntryMut::Vacant(vacant) => {
                let sym = backend.intern(src);
                vacant.insert_hashed_nocheck(hash, sym, ()).0
            }
        };

        *k
    }

    /// Resolves the [symbol](Backend::Symbol) into a reference of T
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
mod test;
