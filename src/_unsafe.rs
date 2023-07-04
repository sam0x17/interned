use std::{
    alloc::Layout,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::datatype::*;
use crate::staticize::*;

#[derive(Copy, Clone)]
pub struct StaticValue {
    ptr: *const (),
    hash: u64,
}

impl StaticValue {
    pub const unsafe fn as_value<'a, T>(&self) -> &'a T {
        &*(self.ptr as *const T)
    }

    pub fn from<T: Hash>(value: T) -> Self {
        Self::with_hash(value, None)
    }

    pub fn with_hash<T: Hash>(value: T, hash: Option<u64>) -> Self {
        let hash = hash.unwrap_or_else(|| {
            let mut hasher = DefaultHasher::default();
            value.hash(&mut hasher);
            hasher.finish()
        });
        let ptr = (Box::leak(Box::from(value)) as *const T) as *const ();
        StaticValue { ptr, hash }
    }
}

impl PartialEq for StaticValue {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for StaticValue {}

impl Hash for StaticValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialOrd for StaticValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.hash.partial_cmp(&other.hash)
    }
}

impl Ord for StaticValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl std::fmt::Debug for StaticValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticValue")
            .field("hash", &self.hash)
            .finish()
    }
}

#[derive(Copy, Clone)]
pub struct StaticSlice {
    ptr: *const (),
    len: usize,
    hash: u64,
}

impl StaticSlice {
    pub unsafe fn as_slice<'a, T>(&self) -> &'a [T] {
        std::slice::from_raw_parts(self.ptr as *const T, self.len)
    }

    pub fn from<T: Hash + Copy>(slice: &[T]) -> Self {
        Self::with_hash(slice, None)
    }

    pub fn with_hash<T: Hash + Copy>(slice: &[T], hash: Option<u64>) -> Self {
        let hash = hash.unwrap_or_else(|| {
            let mut hasher = DefaultHasher::default();
            slice.hash(&mut hasher);
            hasher.finish()
        });
        let ptr = unsafe {
            let ptr = std::alloc::alloc(Layout::array::<T>(slice.len()).unwrap()) as *mut T;
            std::ptr::copy(slice.as_ptr(), ptr, slice.len());
            ptr
        };
        let ptr = (ptr as *const T) as *const ();
        let len = slice.len();
        StaticSlice { ptr, len, hash }
    }
}

impl Hash for StaticSlice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for StaticSlice {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for StaticSlice {}

impl PartialOrd for StaticSlice {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.hash.partial_cmp(&other.hash)
    }
}

impl Ord for StaticSlice {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl std::fmt::Debug for StaticSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticSlice")
            .field("hash", &self.hash)
            .finish()
    }
}

#[derive(Copy, Clone)]
pub struct StaticStr {
    ptr: *const str,
    hash: u64,
}

impl StaticStr {
    pub const unsafe fn as_str<'a>(&self) -> &'a str {
        &*(self.ptr as *const str)
    }

    pub fn from<T: Hash + Copy>(value: &str) -> Self {
        Self::with_hash(value, None)
    }

    pub fn with_hash(value: &str, hash: Option<u64>) -> Self {
        let hash = hash.unwrap_or_else(|| {
            let mut hasher = DefaultHasher::default();
            value.hash(&mut hasher);
            hasher.finish()
        });
        let ptr = Box::leak(Box::from(value)) as *const str;
        let written_value = unsafe { (ptr as *const str).as_ref().unwrap() };
        assert_eq!(written_value, value);
        StaticStr { ptr, hash }
    }
}

impl Hash for StaticStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for StaticStr {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

impl Eq for StaticStr {}

impl PartialOrd for StaticStr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.hash.partial_cmp(&other.hash)
    }
}

impl Ord for StaticStr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl std::fmt::Debug for StaticStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticStr")
            .field("hash", &self.hash)
            .finish()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Static {
    Value(StaticValue),
    Slice(StaticSlice),
    Str(StaticStr),
}

impl Static {
    pub fn as_ptr(&self) -> *const () {
        match self {
            Static::Value(value) => value.ptr,
            Static::Slice(slice) => slice.ptr,
            Static::Str(string) => string.ptr as *const (),
        }
    }

    pub fn hash_code(&self) -> u64 {
        match self {
            Static::Value(value) => value.hash,
            Static::Slice(slice) => slice.hash,
            Static::Str(string) => string.hash,
        }
    }

    pub fn from<T: Hash + Copy>(slice: &[T], hash: Option<u64>) -> Self {
        Static::Slice(StaticSlice::with_hash(slice, hash))
    }

    pub fn from_value<T: Hash>(value: T, hash: Option<u64>) -> Static {
        Static::Value(StaticValue::with_hash(value, hash))
    }

    pub fn from_str(value: &str, hash: Option<u64>) -> Static {
        Static::Str(StaticStr::with_hash(value, hash))
    }

    pub unsafe fn as_slice<'a, T>(&self) -> &'a [T] {
        match self {
            Static::Slice(static_slice) => static_slice.as_slice::<T>(),
            _ => panic!("not a slice type!"),
        }
    }

    pub unsafe fn as_value<'a, T>(&self) -> &'a T {
        match self {
            Static::Value(static_value) => static_value.as_value::<T>(),
            _ => panic!("not a value type!"),
        }
    }

    pub unsafe fn as_str<'a>(&self) -> &'a str {
        match self {
            Static::Str(static_str) => static_str.as_str(),
            _ => panic!("not a &str!"),
        }
    }

    pub unsafe fn _partial_eq<T: PartialEq + DataType + Staticize>(&self, other: &Static) -> bool
    where
        T::SliceValueType: PartialEq,
    {
        self.hash_code() == other.hash_code()
    }

    pub unsafe fn _partial_cmp<T: PartialOrd + Staticize>(
        &self,
        other: &Self,
    ) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Static::Value(a), Static::Value(b)) => {
                a.as_value::<T>().partial_cmp(b.as_value::<T>())
            }
            (Static::Slice(a), Static::Slice(b)) => {
                a.as_slice::<T>().partial_cmp(b.as_slice::<T>())
            }
            (Static::Str(a), Static::Str(b)) => a.as_str().partial_cmp(b.as_str()),
            _ => (static_type_id::<T>(), self.hash_code())
                .partial_cmp(&(static_type_id::<T>(), other.hash_code())),
        }
    }

    pub unsafe fn _cmp<T: Ord + Staticize>(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Static::Value(a), Static::Value(b)) => a.as_value::<T>().cmp(b.as_value::<T>()),
            (Static::Slice(a), Static::Slice(b)) => a.as_slice::<T>().cmp(b.as_slice::<T>()),
            (Static::Str(a), Static::Str(b)) => a.as_str().cmp(b.as_str()),
            _ => (static_type_id::<T>(), self.hash_code())
                .cmp(&(static_type_id::<T>(), other.hash_code())),
        }
    }

    pub unsafe fn _hash<T: Hash + Staticize, H: Hasher>(&self, state: &mut H) {
        let type_id = static_type_id::<T>();
        match self {
            Static::Value(value) => (type_id, value).hash(state),
            Static::Slice(slice) => (type_id, slice).hash(state),
            Static::Str(string) => (type_id, string).hash(state),
        }
    }
}
