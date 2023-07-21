//! Contains some of the unsafe backing types used by interned, mainly [`Static`].

use std::{
    alloc::Layout,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::datatype::*;
use staticize::*;

/// An unsafe internal struct used to represent a type-erased, heap-allocated, static value
/// (i.e. not a reference or slice).
#[derive(Copy, Clone)]
pub struct StaticValue {
    pub ptr: *const (),
    hash: u64,
}

impl StaticValue {
    /// Allows (unsafe) direct access to the value stored in this [`StaticValue`]. Specifying a
    /// `T` that differs from the type of the value actually stored in the [`StaticValue`] is
    /// UB.
    pub const unsafe fn as_value<'a, T>(&self) -> &'a T {
        &*(self.ptr as *const T)
    }

    /// Creates a new [`StaticValue`] from the specified `value`, which must be hashable. Since
    /// [`StaticValue`] does not de-allocate its associated heap value when it is dropped (in
    /// fact, it can't be dropped because it is [`Copy`]), this amounts to a memory leak.
    pub fn from<T: Hash>(value: T) -> Self {
        Self::with_hash(value, None)
    }

    /// Creates a new [`StaticValue`] from the specified `value`, based on a manually-specified
    /// hashcode. Since [`StaticValue`] does not de-allocate its associated heap value when it
    /// is dropped (in fact, it can't be dropped because it is [`Copy`]), this amounts to a
    /// memory leak.
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

/// An unsafe internal struct used to represent a type-erased, heap-allocated, static slice
/// (i.e. not a value or reference).
#[derive(Copy, Clone)]
pub struct StaticSlice {
    pub ptr: *const [()],
    hash: u64,
}

impl StaticSlice {
    /// Allows (unsafe) direct access to the slice stored in this [`StaticSlice`]. Specifying a
    /// `T` that differs from the type of the slice actually stored in the [`StaticSlice`] is
    /// UB.
    pub unsafe fn as_slice<'a, T>(&self) -> &'a [T] {
        std::slice::from_raw_parts(self.ptr as *const T, self.len())
    }

    /// Returns the length of the slice stored in this [`StaticSlice`].
    #[inline]
    pub const fn len(&self) -> usize {
        unsafe { (*self.ptr).len() }
    }

    /// Creates a new [`StaticSlice`] from the specified `slice`, which must be hashable. Since
    /// [`StaticSlice`] does not de-allocate its associated heap slice when it is dropped (in
    /// fact, it can't be dropped because it is [`Copy`]), this amounts to a memory leak.
    pub fn from<T: Hash + Copy>(slice: &[T]) -> Self {
        Self::with_hash(slice, None)
    }

    /// Creates a new [`StaticSlice`] from the specified `slice`, based on a manually-specified
    /// hashcode. Since [`StaticSlice`] does not de-allocate its associated heap value when it
    /// is dropped (in fact, it can't be dropped because it is [`Copy`]), this amounts to a
    /// memory leak.
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
        let ptr = unsafe { std::slice::from_raw_parts(ptr, slice.len()) } as *const [T];
        let ptr = ptr as *const [()];
        StaticSlice { ptr, hash }
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

/// An internal struct used to represent a type-erased, heap-allocated, static string
/// (`&'static str`) (i.e. not a reference, slice, or value).
///
/// [`StaticStr`] is the only variant of [`Static`] where all methods are inherently safe,
/// because no type erasure occurs.
#[derive(Copy, Clone)]
pub struct StaticStr {
    ptr: *const str,
    hash: u64,
}

impl StaticStr {
    /// Allows direct access to the string stored in this [`StaticStr`].
    pub const fn as_str<'a>(&self) -> &'a str {
        unsafe { &*(self.ptr as *const str) }
    }

    /// Creates a new [`StaticStr`] from the specified `&str`. Since [`StaticStr`] does not
    /// de-allocate its associated heap string when it is dropped (in fact, it can't be dropped
    /// because it is [`Copy`]), this amounts to a memory leak.
    pub fn from<T: Hash + Copy>(value: &str) -> Self {
        Self::with_hash(value, None)
    }

    /// Creates a new [`StaticStr`] from the specified `&str`, based on a manually-specified
    /// hashcode. Since [`StaticStr`] does not de-allocate its associated heap string when it
    /// is dropped (in fact, it can't be dropped because it is [`Copy`]), this amounts to a
    /// memory leak.
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

/// An (unsafe) internal enum that generalizes over [`StaticValue`], [`StaticSlice`], and
/// [`StaticStr`].
///
/// Thus [`Static`] represents an arbitrary heap-allocated value with a `'static` lifetime that
/// cannot be dropped/de-allocated.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Static {
    Value(StaticValue),
    Slice(StaticSlice),
    Str(StaticStr),
}

impl Static {
    /// Returns the heap pointer for the data of this [`Static`]. Obtaining the pointer is safe
    /// but doing something with it other than printing it is inherently unsafe.
    pub fn as_ptr(&self) -> *const () {
        match self {
            Static::Value(value) => value.ptr,
            Static::Slice(slice) => slice.ptr as *const (),
            Static::Str(string) => string.ptr as *const (),
        }
    }

    /// Returns the underlying hash code stored in the [`StaticValue`] / [`StaticSlice`] /
    /// [`StaticStr`].
    pub fn hash_code(&self) -> u64 {
        match self {
            Static::Value(value) => value.hash,
            Static::Slice(slice) => slice.hash,
            Static::Str(string) => string.hash,
        }
    }

    /// Creates a [`Static`] from a slice.
    pub fn from<T: Hash + Copy>(slice: &[T], hash: Option<u64>) -> Self {
        Static::Slice(StaticSlice::with_hash(slice, hash))
    }

    /// Creates a [`Static`] from a value.
    pub fn from_value<T: Hash>(value: T, hash: Option<u64>) -> Static {
        Static::Value(StaticValue::with_hash(value, hash))
    }

    /// Creates a [`Static`] from a `&str`.
    pub fn from_str(value: &str, hash: Option<u64>) -> Static {
        Static::Str(StaticStr::with_hash(value, hash))
    }

    /// Unsafely accesses the slice pointed to by the underlying [`StaticSlice`]. If the
    /// underlying variant of the [`Static`] is not a [`StaticSlice`], this method will panic.
    /// Specifying the wrong `T` is UB.
    pub unsafe fn as_slice<'a, T>(&self) -> &'a [T] {
        match self {
            Static::Slice(static_slice) => static_slice.as_slice::<T>(),
            _ => panic!("not a slice type!"),
        }
    }

    /// Unsafely accesses the value pointed to by the underlying [`StaticValue`]. If the
    /// underlying variant of the [`Static`] is not a [`StaticValue`], this method will panic.
    /// Specifying the wrong `T` is UB.
    pub unsafe fn as_value<'a, T>(&self) -> &'a T {
        match self {
            Static::Value(static_value) => static_value.as_value::<T>(),
            _ => panic!("not a value type!"),
        }
    }

    /// Unsafely accesses the `&str` pointed to by the underlying [`StaticStr`]. If the
    /// underlying variant of the [`Static`] is not a [`StaticStr`], this method will panic.
    pub fn as_str<'a>(&self) -> &'a str {
        match self {
            Static::Str(static_str) => static_str.as_str(),
            _ => panic!("not a &str!"),
        }
    }

    /// This is UB if the underlying types differ and a hash collision occurs.
    pub unsafe fn _partial_eq<T: PartialEq + DataType + Staticize>(&self, other: &Static) -> bool
    where
        T::SliceValueType: PartialEq,
    {
        self.hash_code() == other.hash_code()
    }

    /// This is UB if the underlying `T` is specified incorrectly
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
            _ => (T::static_type_id(), self.hash_code())
                .partial_cmp(&(T::static_type_id(), other.hash_code())),
        }
    }

    /// This is UB if the underlying `T` is specified incorrectly
    pub unsafe fn _cmp<T: Ord + Staticize>(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Static::Value(a), Static::Value(b)) => a.as_value::<T>().cmp(b.as_value::<T>()),
            (Static::Slice(a), Static::Slice(b)) => a.as_slice::<T>().cmp(b.as_slice::<T>()),
            (Static::Str(a), Static::Str(b)) => a.as_str().cmp(b.as_str()),
            _ => (T::static_type_id(), self.hash_code())
                .cmp(&(T::static_type_id(), other.hash_code())),
        }
    }

    /// This is UB if the underlying `T` is specified incorrectly
    pub unsafe fn _hash<T: Hash + Staticize, H: Hasher>(&self, state: &mut H) {
        let type_id = T::static_type_id();
        match self {
            Static::Value(value) => (type_id, value).hash(state),
            Static::Slice(slice) => (type_id, slice).hash(state),
            Static::Str(string) => (type_id, string).hash(state),
        }
    }
}
