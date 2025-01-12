use std::ops::Deref;

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

#[test]
fn array_test() {
    let mut interner = Interner::<Box<[u32]>>::default();

    /* TODO: Optimize this mESSSSSS  */
    let v1 = interner.get_or_intern(&Box::from([1,2,3,4]));
    let v2 = interner.get_or_intern(&Box::from([5,4,6,4]));
    let v3 = interner.get_or_intern(&Box::from([1,2,3,4]));

    let v1_res = interner.resolve(v1).unwrap();
    let v2_res = interner.resolve(v2).unwrap();
    let v3_res = interner.resolve(v3).unwrap();

    assert_eq!(v1_res.deref(), &[1,2,3,4]);
    assert_eq!(v2_res.deref(), &[5,4,6,4]);
    assert_eq!(v3_res.deref(), &[1,2,3,4]);
}
