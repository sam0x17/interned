use _unsafe::*;
use interned::*;

use std::hash::Hash;

macro_rules! assert_impl_all {
    ($($typ:ty),* : $($tt:tt)*) => {{
        const fn _assert_impl<T>() where T: $($tt)*, {}
        $(_assert_impl::<$typ>();)*
    }};
}

macro_rules! assert_not_impl_any {
    ($x:ty: $($t:path),+ $(,)?) => {
        const _: fn() = || {
            trait AmbiguousIfImpl<A> {
                fn some_item() {}
            }
            impl<T: ?Sized> AmbiguousIfImpl<()> for T {}
            $({
                #[allow(dead_code)]
                struct Invalid;

                impl<T: ?Sized + $t> AmbiguousIfImpl<Invalid> for T {}
            })+
            let _ = <$x as AmbiguousIfImpl<_>>::some_item;
        };
    };
}

#[test]
fn test_interned_traits() {
    use std::fmt::Debug;
    use std::fmt::Display;

    assert_impl_all!(
        Interned<bool>,
        Interned<usize>,
        Interned<u8>,
        Interned<u16>,
        Interned<u32>,
        Interned<u64>,
        Interned<u128>,
        Interned<i8>,
        Interned<i16>,
        Interned<i32>,
        Interned<i64>,
        Interned<i128>,
        Interned<&str> :
        Copy
            + Clone
            + PartialEq
            + Eq
            + PartialOrd
            + Ord
            + Hash
            + Debug
            + Display
    );

    assert_impl_all!(
        Interned<&[bool]>,
        Interned<&[usize]>,
        Interned<&[u8]>,
        Interned<&[u16]>,
        Interned<&[u32]>,
        Interned<&[u64]>,
        Interned<&[u128]>,
        Interned<&[i8]>,
        Interned<&[i16]>,
        Interned<&[i32]>,
        Interned<&[i64]>,
        Interned<&[i128]> :
        Copy
            + Clone
            + PartialEq
            + Eq
            + PartialOrd
            + Ord
            + Hash
            + Debug
    );

    assert_not_impl_any!(Interned<u8>: Send, Sync);
}

#[test]
fn test_memoized_traits() {
    use std::fmt::Debug;
    use std::fmt::Display;

    assert_impl_all!(
        Memoized<i32, bool>,
        Memoized<i32, usize>,
        Memoized<i32, u8>,
        Memoized<i32, u16>,
        Memoized<i32, u32>,
        Memoized<i32, u64>,
        Memoized<i32, u128>,
        Memoized<i32, i8>,
        Memoized<i32, i16>,
        Memoized<i32, i32>,
        Memoized<i32, i64>,
        Memoized<i32, i128>,
        Memoized<i32, &str> :
        Copy
            + Clone
            + PartialEq
            + Eq
            + PartialOrd
            + Ord
            + Hash
            + Debug
            + Display
    );

    assert_impl_all!(
        Memoized<&str, &[bool]>,
        Memoized<&str, &[usize]>,
        Memoized<&str, &[u8]>,
        Memoized<&str, &[u16]>,
        Memoized<&str, &[u32]>,
        Memoized<&str, &[u64]>,
        Memoized<&str, &[u128]>,
        Memoized<&str, &[i8]>,
        Memoized<&str, &[i16]>,
        Memoized<&str, &[i32]>,
        Memoized<&str, &[i64]>,
        Memoized<&str, &[i128]> :
        Copy
            + Clone
            + PartialEq
            + Eq
            + PartialOrd
            + Ord
            + Hash
            + Debug
    );

    assert_not_impl_any!(Memoized<usize, u8>: Send, Sync);
}

#[test]
fn test_static_alloc() {
    let a = StaticValue::from(37);
    assert_eq!(unsafe { *a.as_value::<i32>() }, 37);
    let b = StaticValue::from(37);
    assert_eq!(a, b); // note: we base equality off of the hash, not the address
    let c = StaticValue::from(8348783947u64);
    assert_ne!(b, c);
    assert_eq!(unsafe { *c.as_value::<u64>() }, 8348783947u64);
}

#[test]
fn test_interned_basics() {
    let initial_interned = num_interned::<i32>();
    let a: Interned<i32> = Interned::from(32);
    let b: Interned<i32> = Interned::from(27);
    assert_ne!(a, b);
    let c: Interned<i32> = Interned::from(32);
    assert_eq!(a, c);
    assert_ne!(b, c);
    assert_eq!(*a.interned_value(), 32);
    assert_eq!(*b.interned_value(), 27);
    assert_eq!(*c.interned_value(), 32);
    assert_eq!(num_interned::<i32>(), initial_interned + 2);
}

#[docify::export]
#[test]
fn test_interned_showcase() {
    let a: Interned<i32> = 1289.into();
    let b = Interned::from(1289);
    let c: Interned<i32> = 47.into();
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(a.as_ptr(), b.as_ptr());
    assert_ne!(b.as_ptr(), c.as_ptr());
    let d: Interned<&str> = "asdf".into();
    assert_ne!(d, "fdsa".into());
    assert_eq!(Interned::from("asdf"), d);
    let e = Interned::from([1, 2, 3, 4, 5].as_slice());
    assert_eq!(e, [1, 2, 3, 4, 5].as_slice().into());
    assert_ne!(e, [4, 1, 7].as_slice().into());
    assert_eq!(format!("{b:?}"), "Interned<i32> { value: 1289 }");
    assert_eq!(format!("{d:?}"), "Interned<&str> { str: \"asdf\" }");
    assert_eq!(e[3], 4);
    assert_eq!(e[0], 1);
    assert_eq!(
        format!("{e:?}"),
        "Interned<&[i32]> { slice: [1, 2, 3, 4, 5] }"
    );
}

#[docify::export]
#[test]
fn test_memoized_showcase() {
    fn expensive_fn(a: usize, b: usize, c: usize) -> String {
        format!("{}", a * a + b * b + c * c)
    }
    let a = Memoized::from((1, 2, 3), |tup: (usize, usize, usize)| {
        expensive_fn(tup.0, tup.1, tup.2).as_str().into()
    });
    assert_eq!(a.as_str(), "14");
}

#[test]
fn test_interned_into() {
    let a: Interned<i32> = 32.into();
    let b = Interned::from(32);
    assert_eq!(a, b);
    let c: Interned<i32> = 43.into();
    assert_ne!(a, c);
    assert_ne!(c, b);
}

#[test]
fn test_interned_str_types() {
    let a: Interned<&str> = Interned::from("this is a triumph");
    let b: Interned<&str> = Interned::from("I'm making a note here: huge success");
    assert_ne!(a, b);
    assert_ne!(a.interned_str(), b.interned_str());
    assert_eq!(a.interned_str(), "this is a triumph");
    assert_eq!(b.interned_str(), "I'm making a note here: huge success");
    let st = String::from("asdf");
    let c = Interned::from(st.as_str());
    let st2 = String::from("asdf");
    let d = Interned::from(st2.as_str());
    assert_eq!(c, d);
    assert_ne!(c, b);
    let st3 = String::from("nope nope");
    let e = Interned::from(st3.as_str());
    assert_ne!(d, e);
    assert_eq!(c.interned_str().as_ptr(), d.interned_str().as_ptr());
}

#[test]
fn test_interned_deref() {
    let a: Interned<i32> = Interned::from(-99);
    assert_eq!(a.interned_value().abs(), 99);
    let b = Interned::from([5, 6, 7].as_slice());
    assert_eq!(b.len(), 3);
    let c = Interned::from("for the good of all of us except the ones who are dead");
    assert_eq!(c.interned_str().chars().next().unwrap(), 'f');
}

#[test]
fn test_memoized_basic() {
    let initial_interned = num_interned::<usize>();
    let initial_memoized = num_memoized::<usize>();
    let a = Memoized::from(&"some_input", |input| input.len().into());
    let b = Memoized::from(&"other", |input| input.len().into());
    assert_ne!(a, b);
    let c = Memoized::from(&"some_input", |input| input.len().into());
    assert_eq!(a, c);
    assert_ne!(b, c);
    assert_eq!(a.as_value(), &10);
    assert_ne!(*a.as_value(), 11);
    assert_eq!(*b.interned().interned_value(), 5);
    assert_eq!(*c.as_value(), 10);
    assert_eq!(num_interned::<usize>(), initial_interned + 2);
    assert_eq!(num_memoized::<usize>(), initial_memoized + 2);
}

#[test]
fn test_interned_byte_arrays() {
    let a: Interned<&[u8]> = Interned::from([1u8, 2u8, 3u8].as_slice());
    let b = Interned::from([5u8, 4u8, 3u8, 2u8, 1u8].as_slice());
    assert_ne!(a.interned_slice().as_ptr(), b.interned_slice().as_ptr());
    let c = Interned::from([1u8, 2u8, 3u8].as_slice());
    assert_eq!(a.interned_slice().as_ptr(), c.interned_slice().as_ptr());
    assert_eq!(a.interned_slice(), c.interned_slice());
    assert_eq!(a, c);
    assert_eq!(c, a);
    assert_ne!(a, b);
    assert_ne!(b, a);
}

#[test]
fn test_static_slice() {
    let slice = &mut [1, 2, 3, 4, 5];
    let a = StaticSlice::from(slice);
    assert_eq!(unsafe { a.as_slice::<i32>() }, &[1, 2, 3, 4, 5]);
    slice[1] = 7;
    assert_eq!(unsafe { a.as_slice::<i32>() }, [1, 2, 3, 4, 5]);
    let b = StaticSlice::from(&[1, 2, 3, 4, 5]);
    assert_eq!(a, b);
    let c = StaticSlice::from(&[true, false, true, false, true, false]);
    assert_ne!(a, c);
    assert_ne!(b, c);
    assert_eq!(
        unsafe { c.as_slice::<bool>() },
        &[true, false, true, false, true, false]
    );
}
