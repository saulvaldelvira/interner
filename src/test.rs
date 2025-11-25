use crate::backend::VecBackend;

use super::*;

use std::collections::HashMap;

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

#[test]
fn box_test() {
    let mut interner = Interner::<[u32]>::default();

    let v1 = interner.get_or_intern(&[1,2,3,4][..]);
    let v2 = interner.get_or_intern(&Vec::from([5,4,6,4]));
    let v3 = interner.get_or_intern(&Box::<[u32]>::from([1, 2, 3, 4]));

    let v1_res = interner.resolve(v1).unwrap();
    let v2_res = interner.resolve(v2).unwrap();
    let v3_res = interner.resolve(v3).unwrap();

    assert_eq!(v1, v3);
    assert_eq!(v1_res, &[1,2,3,4]);
    assert_eq!(v2_res, &[5,4,6,4]);
    assert_eq!(v3_res, &[1,2,3,4]);
}

/// Test that the interner remains coherent after resizing the
/// internal hash table
#[test]
fn resize_test() {
    let mut interner = Interner::<str>::default();

    let mut nums = vec![];
    let mut cap = 0;
    const MAX: usize = 99999;

    for i in 0..MAX {
        let s = format!("{i}");
        let n = interner.get_or_intern(&s);
        nums.push(n);

        /* When half of the elements are inserted,
         * save the capacity for later checking */
        if i == MAX / 2 {
            cap = interner.set.capacity();
        }
    }

    /* The hash map must have been resized */
    assert!(interner.set.capacity() > cap);

    /* All the elements should be resolved correctly */
    for (i,n) in nums.iter().enumerate() {
        let s = format!("{i}");
        assert_eq!(interner.resolve(*n), Some(s.as_str()));

        let new_sym = interner.get_or_intern(&s);
        assert_eq!(new_sym.as_usize(), n.as_usize());
    }

}

#[test]
fn nums() {
    let mut interner = Interner::<i32>::default();

    let a = interner.get_or_intern(&12);

    assert_eq!(a, interner.get_or_intern(&12));

    let n = *interner.resolve(a).unwrap();
    assert_eq!(n, 12);
}

#[test]
fn slices() {
    /* Test that a VecBacked<T> implements Backend<[T]> as well */
    let mut interner = Interner::<[u32], VecBackend<u32>>::new();

    let slice = interner.get_or_intern(&[1, 2, 3, 4, 5]);
    let second = interner.get_or_intern(&[45, 6]);
    let third = interner.get_or_intern(&[1, 2, 3, 4, 5]);

    assert_eq!(slice, third);

    assert_eq!(interner.resolve(slice), Some(&[1, 2, 3, 4, 5][..]));
    assert_eq!(interner.resolve(second), Some(&[45, 6][..]));
}

#[test]
fn from_to_usize() {
    let mut interner = Interner::<str>::default();

    let hello_orig = interner.get_or_intern("Hellooo");

    let hello_num = hello_orig.as_usize();

    let hello_sym = Symbol::<str>::from_usize(hello_num);

    assert_eq!(hello_sym, hello_orig);
}
