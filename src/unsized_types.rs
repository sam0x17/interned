use crate::*;
use core::fmt::Display;
use core::ops::Deref;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

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
/// and [`PartialEq`]/[`Eq`][`PartialOrd`]/[`Ord`] with all other [`OsStr`] and [`OsString`]
/// types, meaning that for the most part you can use an [`InOsStr`] seamlessly in most places
/// where some sort of os string is expected.
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

/// A convenience abstraction around [`Interned<&'static Path>`] with some extra [`From`] impls
/// and other convenience functions. This should be your go-to type if you want to work with
/// interned [`Path`]s and/or [`Path`]s.
///
/// ```
/// use std::path::*;
/// use interned::InPath;
///
/// let a = InPath::from(Path::new("/home/sam"));
/// let b: InPath = Path::new("/home/sam").into();
/// let c: InPath = PathBuf::from(Path::new("/home/sam/Desktop")).into();
/// assert_eq!(a, b);
/// assert_eq!(a, Path::new("/home/sam"));
/// assert_ne!(a, c);
/// assert_ne!(b, c);
/// assert_eq!(a.as_ptr(), b.as_ptr());
/// ```
///
/// Note that as shown above, convenient impls are provided for [`From`]/[`Into`] conversions
/// and [`PartialEq`]/[`Eq`][`PartialOrd`]/[`Ord`] with all other [`Path`] and [`PathBuf`] types,
/// meaning that for the most part you can use an [`InPath`] seamlessly in most places where
/// some sort of string type is expected.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct InPath(Interned<&'static Path>);

impl InPath {
    /// Returns a reference to the underlying interned path for this [`InPath`].
    pub fn as_path(&self) -> &'static Path {
        self.0.interned_path()
    }

    /// Returns the underlying heap pointer where this [`Path`] is stored.
    pub fn as_ptr(&self) -> *const () {
        self.0.as_ptr()
    }
}

impl Display for InPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.interned_path().to_string_lossy())
    }
}

impl AsRef<Path> for InPath {
    fn as_ref(&self) -> &Path {
        self.0.interned_path()
    }
}

impl<'a> From<&'a Path> for InPath {
    fn from(value: &'a Path) -> Self {
        InPath(Interned::<&'static Path>::from(value))
    }
}

impl From<Interned<&'static Path>> for InPath {
    fn from(value: Interned<&'static Path>) -> Self {
        InPath(value)
    }
}

impl<'a> From<InPath> for &'a Path {
    fn from(value: InPath) -> Self {
        value.0.interned_path()
    }
}

impl From<PathBuf> for InPath {
    fn from(value: PathBuf) -> Self {
        InPath::from(value.as_path())
    }
}

impl From<InPath> for PathBuf {
    fn from(value: InPath) -> Self {
        value.as_path().into()
    }
}

impl PartialEq<&Path> for InPath {
    fn eq(&self, other: &&Path) -> bool {
        self.0.interned_path().eq(*other)
    }
}

impl PartialEq<Path> for InPath {
    fn eq(&self, other: &Path) -> bool {
        self.0.interned_path().eq(other)
    }
}

impl PartialOrd<&Path> for InPath {
    fn partial_cmp(&self, other: &&Path) -> Option<std::cmp::Ordering> {
        self.0.interned_path().partial_cmp(*other)
    }
}

impl Deref for InPath {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.0.interned_path()
    }
}

#[test]
fn test_interned_path() {
    let a: Interned<&'static Path> = Path::new("/hey").into();
    let b: &Path = Path::new("/hey");
    assert_eq!(a.interned_path(), b);
}

#[test]
fn test_in_path() {
    let a: InPath = InPath::from(Path::new("/hello/world"));
    let b: InPath = Path::new("hey").into();
    assert_ne!(a, b);
    let c: InPath = Path::new("/hello/world").into();
    assert_eq!(a, c);
    assert_ne!(b, c);
}
