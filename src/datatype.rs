//! Contains the [`DataType`] trait, which is a useful utility trait that provides a number of
//! associated types for `T` such as the internal type in a slice, etc, how `T` should be
//! dereferenced, etc. For a type to work with [`Interned`] and/or [`Memoized`], it must
//! implement [`DataType`].

use crate::_unsafe::*;
use crate::*;
use std::ffi::OsStr;

/// Variant of [`DataTypeTypeMarker`] representing a slice type.
pub enum Slice {}
/// Variant of [`DataTypeTypeMarker`] representing a value type.
pub enum Value {}
/// Variant of [`DataTypeTypeMarker`] representing a reference type.
pub enum Reference {}

/// Used to differentiate between a [`Slice`], [`Value`], and [`Reference`]. This only exists
/// as a trait because enum variants _as types_ are not a thing in Rust presently.
pub trait DataTypeTypeMarker {}

impl DataTypeTypeMarker for Slice {}
impl DataTypeTypeMarker for Value {}
impl DataTypeTypeMarker for Reference {}

/// An (unsafe) trait that must be implemented on any `T` used with [`Interned`] and/or
/// [`Memoized`] that provides utility access to underlying variants of the type.
///
/// Implementers are responsible for accurately implementing each associated type based on its
/// description. Doing so inaccurately (for example, setting `SliceType` to something
/// completely unrelated to `T` when `Type` is [`Slice`]) is UB. Implementers are also
/// responsible for safely implementing the `as_*` methods.
pub unsafe trait DataType {
    /// Specifies whether this type is inherently a [`Slice`], [`Value`], or [`Reference`] (all
    /// types can fit into one of these three categories).
    type Type: DataTypeTypeMarker;

    /// For [`Slice`] types, represents the actual (outer) slice type, which is normally the
    /// same as `T`. For non-[`Slice`] types, this should be set to `()`.
    type SliceType;

    /// For [`Value`] types, specifies the internal type that `T` represents. Typically this is
    /// the same as `T`.
    type ValueType;

    /// For [`Slice`] types, specifies the internal type that `T` is a slice over. Should be
    /// set to `()` for non-[`Slice`] types, particularly unsized slice-like [`Reference`]
    /// types, such as `&str`.
    type SliceValueType;

    /// Represents the "inner type" of this type. For [`Slice`] types, this should be the same
    /// as `SliceValueType`, for [`Value`] types, this should be the same as `T`. For
    /// [`Reference`] types of the form `T = &A`, this should be `A`.
    type InnerType: ?Sized;

    /// Specifies the type that should be used when `T` is dereferenced. This is often the same
    /// as `T` but can differ. For unsized [`Reference] types of the form `T = &A`, this should
    /// be `A`.
    type DerefTargetType: ?Sized;

    /// Accesses `T` as a [`Slice`]. This method should panic if the underlying type cannot be
    /// accessed as a [`Slice`]. Implementers are responsible for handling the unsafe data
    /// access correctly to avoid UB.
    fn as_slice(&self) -> &[Self::SliceValueType];

    /// Accesses `T` as a [`Value`]. This method should panic if the underlying type cannot be
    /// accessed as a [`Value`]. Implementers are responsible for handling the unsafe data
    /// access correctly to avoid UB.
    fn as_value(&self) -> Self::ValueType;

    /// Creates a new [`Static`] from `self` and an optional hash code. A hash code will be
    /// calculated automatically by the machinery in [`Static`] if [`None`] is specified.
    fn to_static_with_hash(&self, hash: Option<u64>) -> Static;

    /// Creates a new [`Static`] from `self`. A hash code will be calculated automatically by
    /// the machinery in [`Static`].
    fn to_static(&self) -> Static {
        self.to_static_with_hash(None)
    }
}

unsafe impl<'a, T: Sized + Hash + Copy> DataType for &'a [T] {
    type Type = Slice;
    type SliceType = &'a [T];
    type ValueType = Self::SliceType;
    type SliceValueType = T;
    type InnerType = T;
    type DerefTargetType = [T];

    fn as_slice(&self) -> &'a [T] {
        *self
    }

    fn as_value(&self) -> &'a [T] {
        *self
    }

    fn to_static_with_hash(&self, hash: Option<u64>) -> Static {
        Static::from(*self, hash)
    }
}

/// A convenience macro provided to easily implement [`DataType`] for [`Value`] types that are
/// neither slices nor references, for which `ValueType`, `InnerType`, and `DerefTargetType`
/// are all equal to `T`.
#[macro_export]
macro_rules! unsafe_impl_data_type {
    ($typ:ty, Value) => {
        unsafe impl $crate::datatype::DataType for $typ {
            type Type = $crate::datatype::Value;
            type SliceType = ();
            type ValueType = $typ;
            type SliceValueType = ();
            type InnerType = $typ;
            type DerefTargetType = $typ;

            fn as_slice(&self) -> &'static [Self::SliceType] {
                panic!("not a slice!");
            }

            fn as_value(&self) -> Self::ValueType {
                *self
            }

            fn to_static_with_hash(&self, hash: Option<u64>) -> $crate::_unsafe::Static {
                $crate::_unsafe::Static::from_value(*self, hash)
            }
        }
    };
}

unsafe impl<'a> DataType for &'a str {
    type Type = Reference;
    type SliceType = &'a str;
    type ValueType = &'a str;
    type SliceValueType = ();
    type InnerType = str;
    type DerefTargetType = str;

    fn as_slice(&self) -> &'static [()] {
        panic!("not supported");
    }

    fn as_value(&self) -> &'a str {
        *self
    }

    fn to_static_with_hash(&self, hash: Option<u64>) -> Static {
        Static::from_str(*self, hash)
    }
}

unsafe impl<'a> DataType for &'a OsStr {
    type Type = Reference;
    type SliceType = &'a OsStr;
    type ValueType = &'a OsStr;
    type SliceValueType = ();
    type InnerType = OsStr;
    type DerefTargetType = OsStr;

    fn as_slice(&self) -> &'static [()] {
        panic!("not supported");
    }

    fn as_value(&self) -> &'a OsStr {
        *self
    }

    fn to_static_with_hash(&self, hash: Option<u64>) -> Static {
        Static::from_os_str(*self, hash)
    }
}

unsafe impl<'a> DataType for &'a Path {
    type Type = Reference;
    type SliceType = &'a Path;
    type ValueType = &'a Path;
    type SliceValueType = ();
    type InnerType = Path;
    type DerefTargetType = Path;

    fn as_slice(&self) -> &'static [()] {
        panic!("not supported");
    }

    fn as_value(&self) -> &'a Path {
        *self
    }

    fn to_static_with_hash(&self, hash: Option<u64>) -> Static {
        Static::from_path(*self, hash)
    }
}

unsafe_impl_data_type!((), Value);
unsafe_impl_data_type!(char, Value);
unsafe_impl_data_type!(bool, Value);
unsafe_impl_data_type!(usize, Value);
unsafe_impl_data_type!(u8, Value);
unsafe_impl_data_type!(u16, Value);
unsafe_impl_data_type!(u32, Value);
unsafe_impl_data_type!(u64, Value);
unsafe_impl_data_type!(u128, Value);
unsafe_impl_data_type!(i8, Value);
unsafe_impl_data_type!(i16, Value);
unsafe_impl_data_type!(i32, Value);
unsafe_impl_data_type!(i64, Value);
unsafe_impl_data_type!(i128, Value);
