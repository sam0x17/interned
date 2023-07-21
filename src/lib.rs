//! This crate provides highly optimized, thread-local, generic
//! [interning](https://en.wikipedia.org/wiki/String_interning) via [`Interned<T>`] and a
//! [memoization](https://en.wikipedia.org/wiki/Memoization) layer built on top of this
//! interning layer, provided by [`Memoized<I, T>`], which can cache the result of an arbitrary
//! input `I: Hash` and _intern_ this result in the underlying interning layer.
//!
//! Blanket implementations supporting `T` are provided for all primitives, slices of [`Sized`]
//! `T` (including `&[u8]`), as well as [`str`] slices (`&str`). Support for additional
//! arbitrary types can be added by implementing [`DataType`], [`Staticize`], and [`Hash`].
//! [`str`] slices have a custom implementation since they are the only built-in unsized type
//! with slice support.
//!
//! All values are heap-allocated `'static`s and benefit from [`TypeId`]-specific locality of
//! reference in the heap. Any two [`Interned<T>`] instances that have the same value of `T`
//! will be guaranteed to point to the same memory address in the heap. Among other things,
//! this allows for `O(1)` (in the size of the data) equality comparisons since the heap
//! addresses are compared instead of having to compare the underlying data bit-by-bit. This
//! makes interned types especially suited for parsing and similar tasks.
//!
//! A caveat of the `'static` lifetime and immutability of the underlying heap data is that
//! unique values of [`Interned<T>`] and [`Memoized<I, T>`] _leak_ in the sense that they can
//! never be de-allocated. This allows us to implement [`Copy`] on all interned types, because
//! we can rely on the heap pointer to continue existing for the life of the program once it
//! has been created for a particular value. For this reason, you should _not_ use this crate
//! for long-running programs that will encounter an unbounded number of unique values, such as
//! those created by an unending stream of user input.
//!
//! Because the internal size of an [`Interned<T>`] _on the stack_ is the size of a [`usize`]
//! (pointer) plus a [`u64`] (cached hash code), it would be silly to use [`Interned<T>`] with
//! integer types directly, however it makes sense to do so for the purposes of memoizing an
//! expensive computation via [`Memoized<I, T>`].
//!
//! ### Interned Example
#![doc = docify::embed_run!("tests/tests.rs", test_interned_showcase)]
//!
//! ### Memoized Example
#![doc = docify::embed_run!("tests/tests.rs", test_memoized_basic)]
//!
//! The following demonstrates how "scopes" work with [`Memoized`]:
#![doc = docify::embed_run!("tests/tests.rs", test_memoized_basic)]

pub mod _unsafe;
pub mod datatype;
pub use datatype::DataType;
pub mod memoized;
pub use memoized::Memoized;

use _unsafe::*;
use datatype::*;
use staticize::*;

use std::{
    any::TypeId,
    cell::RefCell,
    collections::{
        hash_map::{DefaultHasher, Entry},
        HashMap,
    },
    fmt::Display,
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
};

thread_local! {
    static INTERNED: RefCell<HashMap<TypeId, HashMap<u64, Static>, TypeIdHasherBuilder>> = RefCell::new(HashMap::with_hasher(TypeIdHasherBuilder));
    static MEMOIZED: RefCell<HashMap<TypeId, HashMap<u64, Static>, TypeIdHasherBuilder>> = RefCell::new(HashMap::with_hasher(TypeIdHasherBuilder));
}

struct TypeIdHasher {
    hash: Option<u64>,
}

impl Hasher for TypeIdHasher {
    fn finish(&self) -> u64 {
        self.hash.unwrap()
    }

    fn write(&mut self, bytes: &[u8]) {
        debug_assert!(bytes.len() == 8);
        self.hash = Some(bytes.as_ptr() as u64);
    }
}

struct TypeIdHasherBuilder;

impl BuildHasher for TypeIdHasherBuilder {
    type Hasher = TypeIdHasher;

    fn build_hasher(&self) -> Self::Hasher {
        TypeIdHasher { hash: None }
    }
}

#[derive(Copy, Clone)]
pub struct Interned<T: Hash> {
    _value: PhantomData<T>,
    value: Static,
}

impl<T: Hash> Interned<T> {
    pub fn as_ptr(&self) -> *const () {
        self.value.as_ptr()
    }
}

impl<T: Hash + Copy + Staticize + DataType> From<Static> for Interned<T> {
    fn from(value: Static) -> Self {
        let type_id = T::static_type_id();
        let entry = INTERNED.with(|interned| {
            *interned
                .borrow_mut()
                .entry(type_id)
                .or_insert_with(|| HashMap::new())
                .entry(value.hash_code())
                .or_insert(value)
        });
        Interned {
            _value: PhantomData,
            value: entry,
        }
    }
}

impl<T: Hash + Copy + Staticize + DataType + From<Interned<T>>> From<T> for Interned<T::Static>
where
    <T as Staticize>::Static: Hash + Sized,
{
    fn from(value: T) -> Interned<T::Static> {
        let mut hasher = DefaultHasher::default();
        value.hash(&mut hasher);
        let hash = hasher.finish();
        let type_id = T::static_type_id();
        let entry = INTERNED.with(|interned| {
            *interned
                .borrow_mut()
                .entry(type_id)
                .or_insert_with(|| HashMap::new())
                .entry(hash)
                .or_insert_with(|| value.to_static_with_hash(Some(hash)))
        });
        Interned {
            _value: PhantomData,
            value: entry,
        }
    }
}

impl<T: Hash + Staticize + DataType<Type = Slice>> Interned<T> {
    pub fn interned_slice<'a>(&self) -> &'a [T::SliceValueType] {
        unsafe { self.value.as_slice::<T::SliceValueType>() }
    }
}

impl Interned<&str> {
    pub fn interned_str<'a>(&self) -> &'a str {
        unsafe { self.value.as_str() }
    }
}

impl<T: Hash + Staticize + DataType<Type = Value>> Interned<T> {
    pub fn interned_value<'a>(&self) -> &'a T {
        unsafe { self.value.as_value() }
    }
}

impl<T: Hash + Staticize + DataType> Deref for Interned<T> {
    type Target = T::DerefTargetType;

    fn deref(&self) -> &Self::Target {
        match self.value {
            Static::Slice(static_slice) => unsafe {
                let target_ref: &[T::SliceValueType] =
                    &*(static_slice.ptr as *const [T::SliceValueType]);
                std::mem::transmute_copy(&target_ref)
            },
            Static::Value(static_value) => unsafe {
                std::mem::transmute_copy(&static_value.as_value::<T>())
            },
            Static::Str(static_str) => unsafe { std::mem::transmute_copy(&static_str.as_str()) },
        }
    }
}

impl<T: Hash + PartialEq + Staticize + DataType> PartialEq for Interned<T>
where
    <T as DataType>::SliceValueType: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.value._partial_eq::<T>(&other.value) }
    }
}

impl<T: Hash + Staticize + Eq + DataType> Eq for Interned<T>
where
    T: PartialEq,
    <T as DataType>::SliceValueType: PartialEq,
{
}

impl<T: Hash + Staticize + PartialOrd + DataType> PartialOrd for Interned<T>
where
    <T as DataType>::SliceValueType: PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        unsafe { self.value._partial_cmp::<T>(&other.value) }
    }
}

impl<T: Hash + Staticize + Ord + DataType> Ord for Interned<T>
where
    <T as DataType>::SliceValueType: PartialEq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        unsafe { self.value._cmp::<T>(&other.value) }
    }
}

impl<T: Hash + Staticize> Hash for Interned<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { self.value._hash::<T, H>(state) }
    }
}

impl<T: Hash + Staticize + DataType + std::fmt::Debug> std::fmt::Debug for Interned<T>
where
    <T as DataType>::SliceValueType: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct(format!("Interned<{}>", T::static_type_name()).as_str());
        let ret = match self.value {
            Static::Value(value) => f.field("value", unsafe { value.as_value::<T>() }),
            Static::Slice(slice) => {
                f.field("slice", unsafe { &slice.as_slice::<T::SliceValueType>() })
            }
            Static::Str(string) => f.field("str", unsafe { &string.as_str() }),
        }
        .finish();
        ret
    }
}

impl<T: Hash + Display> Display for Interned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Pointer;
        match self.value {
            Static::Value(value) => unsafe { value.as_value::<T>().fmt(f) },
            Static::Slice(slice) => unsafe { slice.as_slice::<T>().fmt(f) },
            Static::Str(string) => unsafe { string.as_str().fmt(f) },
        }
    }
}

pub fn num_memoized<T: Staticize>() -> usize {
    let type_id = T::static_type_id();
    MEMOIZED.with(|interned| interned.borrow_mut().entry(type_id).or_default().len())
}

pub fn num_interned<T: Staticize>() -> usize {
    let type_id = T::static_type_id();
    INTERNED.with(|interned| interned.borrow_mut().entry(type_id).or_default().len())
}

macro_rules! derive_from_interned_impl_value {
    ($ty:ty) => {
        impl From<$crate::Interned<$ty>> for $ty {
            fn from(value: Interned<$ty>) -> Self {
                use $crate::_unsafe::Static::*;
                match value.value {
                    Value(val) => unsafe { *val.as_value() },
                    _ => unreachable!(),
                }
            }
        }
    };
}

macro_rules! derive_from_interned_impl_slice {
    ($ty:ty) => {
        impl From<$crate::Interned<$ty>> for $ty {
            fn from(value: Interned<$ty>) -> Self {
                use $crate::_unsafe::Static::*;
                match value.value {
                    Slice(slice) => unsafe { slice.as_slice() },
                    _ => unreachable!(),
                }
            }
        }
    };
}

impl From<Interned<&str>> for &str {
    fn from(value: Interned<&str>) -> Self {
        value.interned_str()
    }
}

derive_from_interned_impl_value!(char);
derive_from_interned_impl_value!(bool);
derive_from_interned_impl_value!(usize);
derive_from_interned_impl_value!(u8);
derive_from_interned_impl_value!(u16);
derive_from_interned_impl_value!(u32);
derive_from_interned_impl_value!(u64);
derive_from_interned_impl_value!(u128);
derive_from_interned_impl_value!(i8);
derive_from_interned_impl_value!(i16);
derive_from_interned_impl_value!(i32);
derive_from_interned_impl_value!(i64);
derive_from_interned_impl_value!(i128);
derive_from_interned_impl_slice!(&[bool]);
derive_from_interned_impl_slice!(&[usize]);
derive_from_interned_impl_slice!(&[u8]);
derive_from_interned_impl_slice!(&[u16]);
derive_from_interned_impl_slice!(&[u32]);
derive_from_interned_impl_slice!(&[u64]);
derive_from_interned_impl_slice!(&[u128]);
derive_from_interned_impl_slice!(&[i8]);
derive_from_interned_impl_slice!(&[i16]);
derive_from_interned_impl_slice!(&[i32]);
derive_from_interned_impl_slice!(&[i64]);
derive_from_interned_impl_slice!(&[i128]);
