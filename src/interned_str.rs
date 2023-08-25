use crate::*;
use core::fmt::Display;
use core::ops::Deref;
use std::ffi::OsString;

/// A convenience abstraction around [`Interned<&'static str>`] with some extra [`From`] impls
/// and other convenience functions. This should be your go-to type if you want to work with
/// interned strings.
///
/// ```
/// use interned::InStr;
///
/// let a = InStr::from("this is a triumph");
/// let b: InStr = String::from("this is a triumph").into();
/// let c: InStr = "I'm making a note here, huge success".into();
/// assert_eq!(a, b);
/// assert_eq!(a, "this is a triumph");
/// assert_ne!(a, c);
/// assert_ne!(b, c);
/// assert_eq!(a.as_ptr(), b.as_ptr());
/// ```
///
/// Note that as shown above, convenient impls are provided for [`From`]/[`Into`] conversions
/// and [`PartialEq`]/[`Eq`][`PartialOrd`]/[`Ord`] with all other [`str`] and [`String`] types,
/// meaning that for the most part you can use an [`InStr`] seamlessly in most places where
/// some sort of string type is expected.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct InStr(Interned<&'static str>);

impl InStr {
    /// Returns a reference to the underlying interned string for this [`InStr`].
    pub fn as_str(&self) -> &'static str {
        self.0.interned_str()
    }

    /// Returns the underlying heap pointer where this [`str`] is stored.
    pub fn as_ptr(&self) -> *const () {
        self.0.as_ptr()
    }
}

impl Display for InStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.interned_str())
    }
}

impl AsRef<str> for InStr {
    fn as_ref(&self) -> &str {
        self.0.interned_str()
    }
}

impl<'a> From<&'a str> for InStr {
    fn from(value: &'a str) -> Self {
        InStr(Interned::<&'static str>::from(value))
    }
}

impl From<String> for InStr {
    fn from(value: String) -> Self {
        InStr(Interned::<&'static str>::from(value.as_str()))
    }
}

impl From<Interned<&'static str>> for InStr {
    fn from(value: Interned<&'static str>) -> Self {
        InStr(value)
    }
}

impl<'a> From<InStr> for &'a str {
    fn from(value: InStr) -> Self {
        value.0.interned_str()
    }
}

impl From<InStr> for String {
    fn from(value: InStr) -> Self {
        value.0.interned_str().to_string()
    }
}

impl PartialEq<&str> for InStr {
    fn eq(&self, other: &&str) -> bool {
        self.0.interned_str().eq(*other)
    }
}

impl PartialEq<String> for InStr {
    fn eq(&self, other: &String) -> bool {
        self.0.interned_str().eq(other.as_str())
    }
}

impl PartialOrd<&str> for InStr {
    fn partial_cmp(&self, other: &&str) -> Option<std::cmp::Ordering> {
        self.0.interned_str().partial_cmp(*other)
    }
}

impl Deref for InStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.interned_str()
    }
}

/// A convenience abstraction around [`Interned<&'static OsStr>`] with some extra [`From`] impls
/// and other convenience functions. This should be your go-to type if you want to work with
/// interned [`OsStr`]s and/or [`OsString`]s.
///
/// ```
/// use std::ffi::{OsStr, OsString};
/// use interned::InOsStr;
///
/// let a = InOsStr::from(OsStr::new("this is a triumph"));
/// let b: InOsStr = OsString::from("this is a triumph").into();
/// let c: InOsStr = OsStr::new("I'm making a note here, huge success").into();
/// assert_eq!(a, b);
/// assert_eq!(a, OsStr::new("this is a triumph"));
/// assert_ne!(a, c);
/// assert_ne!(b, c);
/// assert_eq!(a.as_ptr(), b.as_ptr());
/// ```
///
/// Note that as shown above, convenient impls are provided for [`From`]/[`Into`] conversions
/// and [`PartialEq`]/[`Eq`][`PartialOrd`]/[`Ord`] with all other [`str`] and [`String`] types,
/// meaning that for the most part you can use an [`InOsStr`] seamlessly in most places where
/// some sort of string type is expected.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct InOsStr(Interned<&'static OsStr>);

impl InOsStr {
    /// Returns a reference to the underlying interned string for this [`InOsStr`].
    pub fn as_os_str(&self) -> &'static OsStr {
        self.0.interned_os_str()
    }

    /// Returns the underlying heap pointer where this [`OsStr`] is stored.
    pub fn as_ptr(&self) -> *const () {
        self.0.as_ptr()
    }
}

impl Display for InOsStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.interned_os_str().to_string_lossy())
    }
}

impl AsRef<OsStr> for InOsStr {
    fn as_ref(&self) -> &OsStr {
        self.0.interned_os_str()
    }
}

impl<'a> From<&'a OsStr> for InOsStr {
    fn from(value: &'a OsStr) -> Self {
        InOsStr(Interned::<&'static OsStr>::from(value))
    }
}

impl From<OsString> for InOsStr {
    fn from(value: OsString) -> Self {
        InOsStr(Interned::<&'static OsStr>::from(value.as_os_str()))
    }
}

impl From<Interned<&'static OsStr>> for InOsStr {
    fn from(value: Interned<&'static OsStr>) -> Self {
        InOsStr(value)
    }
}

impl<'a> From<InOsStr> for &'a OsStr {
    fn from(value: InOsStr) -> Self {
        value.0.interned_os_str()
    }
}

impl From<InOsStr> for OsString {
    fn from(value: InOsStr) -> Self {
        value.0.interned_os_str().to_os_string()
    }
}

impl PartialEq<&OsStr> for InOsStr {
    fn eq(&self, other: &&OsStr) -> bool {
        self.0.interned_os_str().eq(*other)
    }
}

impl PartialEq<OsString> for InOsStr {
    fn eq(&self, other: &OsString) -> bool {
        self.0.interned_os_str().eq(other.as_os_str())
    }
}

impl PartialOrd<&OsStr> for InOsStr {
    fn partial_cmp(&self, other: &&OsStr) -> Option<std::cmp::Ordering> {
        self.0.interned_os_str().partial_cmp(*other)
    }
}

impl Deref for InOsStr {
    type Target = OsStr;

    fn deref(&self) -> &Self::Target {
        self.0.interned_os_str()
    }
}

#[test]
fn test_interned_os_str() {
    let a: Interned<&'static OsStr> = OsStr::new("hey").into();
    let b: &OsStr = OsStr::new("hey");
    assert_eq!(a.interned_os_str(), b);
}

#[test]
fn test_in_os_str() {
    let a: InOsStr = InOsStr::from(OsStr::new("hello world"));
    let b: InOsStr = OsString::from("hey").into();
    assert_ne!(a, b);
    let c: InOsStr = OsStr::new("hello world").into();
    assert_eq!(a, c);
    assert_ne!(b, c);
}
