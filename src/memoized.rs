//! Home of the [`Memoized`] struct, which provides a memoization facility on top of
//! [`Interned`] so you can memoize and intern values based on _input_ and a specified _scope_.

use crate::*;

/// Allows you to memoize a value in [`Interned`] storage based on a specified _input_,
/// _generator_, and _scope_.
///
/// [`Memoized`] takes two generic types, `T`, which is the underlying datatype being stored,
/// and `I`, which is the _input_ type used to generate the `T` value.
///
/// To memoize a value, you must use [`Memoized::from`].
///
/// The following example showcases some general usage for [`Memoized`]:
#[doc = docify::embed_run!("tests/tests.rs", test_memoized_basic)]
#[derive(Copy, Clone)]
pub struct Memoized<I: Hash, T: Hash + Staticize + DataType> {
    _input: PhantomData<I>,
    #[doc(hidden)]
    pub interned: Interned<T>,
}

impl<I: Hash, T: Hash + Staticize + DataType> Memoized<I, T> {
    /// Returns the underlying [`Interned`] stored by this [`Memoized`].
    #[inline]
    pub fn interned(&self) -> Interned<T> {
        Interned {
            _value: PhantomData,
            value: self.interned.value,
        }
    }
}

impl<I: Hash, T: Hash + Staticize + DataType<Type = Slice>> Memoized<I, T> {
    /// Accesses the underlying memoized value as a slice. This will panic if the value is not
    /// a slice.
    pub fn as_slice<'a>(&self) -> &'a [T::SliceValueType] {
        unsafe { self.interned.value.as_slice::<T::SliceValueType>() }
    }
}

impl<I: Hash> Memoized<I, &str> {
    /// Accesses the underlying memoized value as a `&'static str`. This will panic if the
    /// value is not a `&str`.
    pub fn as_str<'a>(&self) -> &'a str {
        self.interned.value.as_str()
    }
}

impl<I: Hash, T: Hash + Staticize + DataType<Type = Value>> Memoized<I, T> {
    /// Accesses the underlying memoized value as a (referenced) value. This will panic if the
    /// value is actually a slice or `&str`.
    pub fn as_value<'a>(&self) -> &'a T {
        unsafe { self.interned.value.as_value() }
    }
}

impl<I: Hash, T: Hash + Copy + Staticize + DataType> Memoized<I, T>
where
    T::Static: Hash + Copy + Clone + DataType,
{
    /// Memoizes the provided `generator` closure/function for the specified unique `scope`
    /// (which can be any hashable value that uniquely identifies the context of this generator).
    ///
    /// The memoized value is produced by hashing the `input`, `scope` and the [`TypeId`] of `T` together and,
    /// checking the thread-local memoized heap storage to see if a value is already memoized
    /// for this combination of `input` + `scope` + `T`. If it is, an instance of [`Memoized`]
    /// is created and returned referencing this heap value. If this combination isn't already
    /// memoized, `input` is fed into `generator` to produce the output value (of type `T`),
    /// and this value is then interned and a [`Memoized`] instance referencing it is returned.
    /// If the value happens to already exist in [`Interned`]'s storage, this existing
    /// [`Interned`] will be automatically used.
    ///
    /// Thus [`Memoized`] provides perfect memory de-duplication for all memoized values.
    ///
    #[doc = docify::embed_run!("tests/tests.rs", test_memoized_showcase)]
    pub fn from<S, G>(scope: S, input: I, generator: G) -> Memoized<I, T>
    where
        S: Hash,
        G: Fn(I) -> Interned<T>,
    {
        let mut hasher = DefaultHasher::default();
        let type_id = T::static_type_id();
        input.hash(&mut hasher);
        scope.hash(&mut hasher);
        type_id.hash(&mut hasher);
        let input_hash = hasher.finish();
        let value_static = MEMOIZED.with(|memoized| {
            match (*memoized)
                .borrow_mut()
                .entry(type_id)
                .or_insert_with(|| HashMap::new())
                .entry(input_hash)
            {
                Entry::Occupied(entry) => *entry.get(),
                Entry::Vacant(entry) => *entry.insert(generator(input).value),
            }
        });
        Memoized {
            _input: PhantomData,
            interned: value_static.into(),
        }
    }
}

impl<I: Hash, T: Hash + Staticize + DataType> Deref for Memoized<I, T> {
    type Target = T::DerefTargetType;

    fn deref(&self) -> &Self::Target {
        match self.interned.value {
            Static::Slice(static_slice) => unsafe {
                let target_ref: &[T::SliceValueType] =
                    &*(static_slice.ptr as *const [T::SliceValueType]);
                std::mem::transmute_copy(&target_ref)
            },
            Static::Value(static_value) => unsafe {
                std::mem::transmute_copy(&static_value.as_value::<T>())
            },
            Static::Str(static_str) => unsafe { std::mem::transmute_copy(&static_str.as_str()) },
            Static::OsStr(static_os_str) => unsafe {
                std::mem::transmute_copy(&static_os_str.as_os_str())
            },
            Static::Path(static_path) => unsafe {
                std::mem::transmute_copy(&static_path.as_path())
            },
        }
    }
}

impl<I: Hash, T: Hash + PartialEq + Staticize + DataType> PartialEq for Memoized<I, T>
where
    <T as DataType>::SliceValueType: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.interned() == other.interned()
    }
}

impl<I: Hash, T: Hash + Eq + Staticize + DataType> Eq for Memoized<I, T> where
    <T as DataType>::SliceValueType: PartialEq
{
}

impl<I: Hash, T: Hash + PartialOrd + Staticize + DataType> PartialOrd for Memoized<I, T>
where
    <T as DataType>::SliceValueType: PartialEq,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.interned().partial_cmp(&other.interned())
    }
}

impl<I: Hash, T: Hash + Ord + Staticize + DataType> Ord for Memoized<I, T>
where
    <T as DataType>::SliceValueType: PartialEq,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.interned().cmp(&other.interned())
    }
}

impl<I: Hash, T: Hash + Staticize + DataType> Hash for Memoized<I, T>
where
    <T as DataType>::SliceValueType: PartialEq,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.interned().hash(state)
    }
}

impl<I: Hash, T: Hash + Staticize + DataType + std::fmt::Debug> std::fmt::Debug for Memoized<I, T>
where
    <T as DataType>::SliceValueType: PartialEq,
    <T as DataType>::SliceValueType: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Memoized")
            .field("interned_value", &self.interned())
            .finish()
    }
}

impl<I: Hash, T: Hash + Staticize + DataType + Display> Display for Memoized<I, T>
where
    <T as DataType>::SliceValueType: PartialEq,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.interned().fmt(f)
    }
}
