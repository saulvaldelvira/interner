/*  Copyright (C) 2025 Saúl Valdelvira
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, version 3.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>. */

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

use hashbrown::hash_map::RawEntryMut;
use hashbrown::HashMap;
use core::borrow::Borrow;
use std::hash::{BuildHasher, Hash, RandomState};

pub mod backend;
pub use backend::{Backend, DefaultBackendBuilder, StringBackend};

use crate::backend::Internable;

pub type Symbol<T, B = <T as DefaultBackendBuilder>::Backend> = <B as Backend<T>>::Symbol;

pub type StringInterner = Interner<str,StringBackend>;

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
    set: HashMap<B::Symbol, (), ()>,
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
    pub const fn with_backend_and_hasher(backend: B, hasher: H) -> Self {
        Self {
            backend,
            hasher,
            set: HashMap::with_hasher(()),
        }
    }

    /// Gets the [Symbol](Backend::Symbol) for `src`, interning it if it doesn't exist.
    ///
    /// # Example
    /// ```
    /// use interns::Interner;
    ///
    /// let mut interner = Interner::<str>::new();
    /// let name = interner.get_or_intern("Abcd");
    /// let name_again = interner.get_or_intern("Abcd");
    /// assert_eq!(name, name_again);
    /// ```
    pub fn get_or_intern<Ref>(&mut self, src: &Ref) -> B::Symbol
    where
        Ref: Internable<T, B> + ?Sized + Hash + Eq,
        T: Borrow<Ref>,
    {
        /* We are doing shenanigans here.
         *
         * We are storing B::Symbol as the key, but we don't hash the
         * Symbol itself. `src` is a reference to T, so we have no way
         * of getting a Symbol from `src`.
         *
         * When we look for a symbol, we pass the hash or `src`, and also provide
         * a custom function to check if the keys match (in case of collision).
         * This function must resolve the Symbol to a value of T, and test it against `src`.
         *
         * When we insert a new element, we also need to provide a custom hasher function.
         * This is because an insertion could cause the table to resize, thus causing all
         * keys to be rehashed. If we don't provide a custom hasher function, a resize
         * would reallocate the keys according to the Symbol, not the `T` value they
         * resolve to, making it imposible to retrive those symbols from a `T` reference
         * in the future.
         *
         * For this trick to work, we need to make sure that we _always_ access the table
         * with a custom function, that resolves the Symbols before hashing/comparing.
         */

        let Self {
            backend,
            set,
            hasher,
        } = self;

        let hash = hasher.hash_one(src);

        let entry = set
            .raw_entry_mut()
            .from_hash(hash, |&sym| {
                /* SAFETY: If the symbol is on the table it must also be on the backend. */
                src == unsafe { backend.get_unchecked(sym) }.borrow()
            });

        let k = match entry {
            RawEntryMut::Occupied(occupied) => occupied.into_key(),
            RawEntryMut::Vacant(vacant) => {
                let sym = backend.intern(src);
                vacant
                    .insert_with_hasher(hash, sym, (), |sym| {
                        /* SAFETY: We've interned the symbol on the call to `Backed::intern` above */
                        let src = unsafe { backend.get_unchecked(*sym) };
                        hasher.hash_one(src)
                    })
                    .0
            }
        };

        *k
    }

    /// Resolves the [symbol](Backend::Symbol) into a reference of T
    ///
    /// # Example
    /// ```
    /// use interns::Interner;
    ///
    /// let mut interner = Interner::<str>::new();
    /// let name = interner.get_or_intern("Abcd");
    /// let resolved = interner.resolve(name);
    /// assert_eq!(resolved, Some("Abcd"));
    /// ```
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
