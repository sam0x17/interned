use crate::_unsafe::*;
use crate::*;

pub enum Slice {}
pub enum Value {}
pub enum Reference {}

pub trait DataTypeTypeMarker {}
impl DataTypeTypeMarker for Slice {}
impl DataTypeTypeMarker for Value {}
impl DataTypeTypeMarker for Reference {}

pub trait DataType {
    type Type: DataTypeTypeMarker;
    type SliceType;
    type ValueType;
    type SliceValueType;
    type InnerType: ?Sized;
    type DerefType;

    fn as_slice(&self) -> &[Self::SliceValueType];
    fn as_value(&self) -> Self::ValueType;
    fn to_static_with_hash(&self, hash: Option<u64>) -> Static;

    fn to_static(&self) -> Static {
        self.to_static_with_hash(None)
    }
}

impl<'a, T: Sized + Hash + Copy> DataType for &'a [T] {
    type Type = Slice;
    type SliceType = &'a [T];
    type ValueType = Self::SliceType;
    type SliceValueType = T;
    type InnerType = T;
    type DerefType = &'a [T];

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

#[macro_export]
macro_rules! impl_data_type {
    ($typ:ty, Value) => {
        impl $crate::datatype::DataType for $typ {
            type Type = $crate::datatype::Value;
            type SliceType = ();
            type ValueType = $typ;
            type SliceValueType = ();
            type InnerType = $typ;
            type DerefType = $typ;

            fn as_slice(&self) -> &'static [Self::SliceType] {
                panic!("not a slice!");
            }

            fn as_value(&self) -> Self::ValueType {
                *self
            }

            fn to_static_with_hash(&self, hash: Option<u64>) -> Static {
                Static::from_value(*self, hash)
            }
        }
    };
}

impl<'a> DataType for &'a str {
    type Type = Reference;
    type SliceType = &'a str;
    type ValueType = &'a str;
    type SliceValueType = ();
    type InnerType = str;
    type DerefType = &'a str;

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

impl_data_type!(bool, Value);
impl_data_type!(usize, Value);
impl_data_type!(u8, Value);
impl_data_type!(u16, Value);
impl_data_type!(u32, Value);
impl_data_type!(u64, Value);
impl_data_type!(u128, Value);
impl_data_type!(i8, Value);
impl_data_type!(i16, Value);
impl_data_type!(i32, Value);
impl_data_type!(i64, Value);
impl_data_type!(i128, Value);
