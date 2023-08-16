[![Crates.io](https://img.shields.io/crates/v/interned)](https://crates.io/crates/interned)
[![docs.rs](https://img.shields.io/docsrs/interned?label=docs)](https://docs.rs/interned/latest/interned/)
[![Build Status](https://img.shields.io/github/actions/workflow/status/sam0x17/interned/ci.yaml)](https://github.com/sam0x17/interned/actions/workflows/ci.yaml?query=branch%3Amain)
[![MIT License](https://img.shields.io/github/license/sam0x17/interned)](https://github.com/sam0x17/interned/blob/main/LICENSE)

Interned provides highly optimized, thread-local, generic
[interning](https://en.wikipedia.org/wiki/String_interning) via `Interned<T>` and a
[memoization](https://en.wikipedia.org/wiki/Memoization) layer built on top of this interning
layer, provided by `Memoized<I, T>`, which can cache the result of an arbitrary input `I: Hash`
and _intern_ this result in the underlying interning layer.

Blanket implementations supporting `T` are provided for all primitives, slices of `Sized` `T`
(including `&[u8]`), as well as `str` slices (`&str`). Support for additional arbitrary types
can be added by implementing `DataType`, `Staticize`, and `Hash`. `str` slices have a custom
implementation since they are the only built-in unsized type with slice support.

All values are heap-allocated `'static`s and benefit from `TypeId`-specific locality of
reference in the heap. Any two `Interned<T>` instances that have the same value of `T` will be
guaranteed to point to the same memory address in the heap. Among other things, this allows for
`O(1)` (in the size of the data) equality comparisons since the heap addresses are compared
instead of having to compare the underlying data bit-by-bit. This makes interned types
especially suited for parsing and similar low-entropy data tasks.

A caveat of the `'static` lifetime and immutability of the underlying heap data is that unique
values of `Interned<T>` and `Memoized<I, T>` _leak_ in the sense that they can never be
de-allocated. This allows us to implement `Copy` on all interned types, because we can rely on
the heap pointer to continue existing for the life of the program once it has been created for
a particular value. For this reason, you should _not_ use this crate for long-running programs
that will encounter an unbounded number of unique values, such as those created by an unending
stream of user input.

Because the internal size of an `Interned<T>` _on the stack_ is the size of a `usize` (pointer)
plus a `u64` (cached hash code), it would be silly to use `Interned<T>` with integer types
directly, however it makes sense to do so for the purposes of memoizing an expensive
computation via `Memoized<I, T>`.

An interned string type, `InStr`, is also provided as a convenient wrapper around
`Interned<&'static str>`. It has a number of extra impls and should be your go-to type if you
want to work with interned strings.

### Interned Example
<!-- docify::embed!("tests/tests.rs", test_interned_showcase) -->

### Memoized Example
<!-- docify::embed!("tests/tests.rs", test_memoized_basic) -->

The following demonstrates how "scopes" work with `Memoized`:
<!-- docify::embed!("tests/tests.rs", test_memoized_basic) -->
