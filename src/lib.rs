pub mod _unsafe;
pub mod staticize;
pub use staticize::Staticize;
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
        let type_id = static_type_id::<T>();
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

impl<T: Hash + Copy + Staticize + DataType> From<T> for Interned<T::Static>
where
    <T as Staticize>::Static: Hash + Sized,
{
    fn from(value: T) -> Interned<T::Static> {
        let mut hasher = DefaultHasher::default();
        value.hash(&mut hasher);
        let hash = hasher.finish();
        let type_id = static_type_id::<T>();
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

impl<T: Hash + Staticize + DataType<Type = Slice>> Deref for Interned<T> {
    type Target = [T::SliceValueType];

    fn deref(&self) -> &Self::Target {
        match self.value {
            Static::Slice(static_slice) => unsafe { static_slice.as_slice() },
            _ => unreachable!(),
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
        let mut f = f.debug_struct(format!("Interned<{}>", static_type_name::<T>()).as_str());
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
    let type_id = static_type_id::<T>();
    MEMOIZED.with(|interned| interned.borrow_mut().entry(type_id).or_default().len())
}

pub fn num_interned<T: Staticize>() -> usize {
    let type_id = static_type_id::<T>();
    INTERNED.with(|interned| interned.borrow_mut().entry(type_id).or_default().len())
}
