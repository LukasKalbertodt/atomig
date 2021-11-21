//! Generic and convenient `std` atomics.
//!
//! This crate offers the generic [`Atomic<T>`][Atomic] type which can perform
//! atomic operations on `T`. There is an important difference to C++'s
//! `atomic` and the `Atomic` type from the `atomic` crate: **the
//! `Atomic<T>` in this crate only works with types that actually support
//! atomic operations on the target platform**. A lock-based fallback for other
//! types is not used!
//!
//! This crate uses the atomic types from `std::sync::atomic` under the hood
//! and actually does not contain any "interesting" runtime code itself. In
//! other words: this is just a nicer API. Thanks to this, this crate does not
//! use any `unsafe` code!
//!
//!
//! # Quick example
//!
//! You can simply use `Atomic<T>` with all types that implement [`Atom`] which
//! are all types that support atomic operations on your platform, but also
//! types that can be represented as the former kind of types (like `f32` and
//! `char`).
//!
//! ```
//! use atomig::{Atomic, Ordering};
//!
//! let a = Atomic::new(true);  // Atomic<bool>
//! a.store(false, Ordering::SeqCst);
//! ```
//!
//! The interface of [`Atomic`] very closely matches the interface of the
//! atomic types in `std::sync::atomic` and you should be able to use this
//! crate as a drop-in replacement. For more examples, see the `examples/`
//! folder in the repository.
//!
//! As you can see in the example, `Ordering` (from `std::sync::atomic`) is
//! reexported in this crate for your import convenience.
//!
//!
//! # Traits
//!
//! This crate contains three main traits to safely abstract over different
//! atomic types. There are also three traits for plumbing in the `impls`
//! module but these are considered implementation detail for the most part and
//! can usually be ignored.
//!
//! The most important trait is probably [`Atom`]: to use a type `T` in
//! `Atomic<T>`, is has to implement [`Atom`]. You can implement that trait for
//! your own types as long as they can be represented by a type that implements
//! [`impls::PrimitiveAtom`]. In many cases, you can also simply
//! `#[derive(Atom)]` for your own types. See [`Atom`]'s documentation for more
//! information.
//!
//!
//! # Notes
//!
//! Not all types and modules from `std::sync::atomic` are available on all
//! platforms. `std` itself uses `cfg(target_has_atomic)` attributes to do
//! that. Unfortunately, these `cfg` attributes are still unstable, thus
//! `atomig` does not use them right now. This has the unfortunate effect that
//! this whole library will fail to compile on any targets where any of the
//! atomic methods/types are unavailable. Once this is stabilized, `atomic`
//! will use those `cfg` attributes as appropriate to support platforms with
//! partial atomic support.
//!
//! # Cargo features
//!
//! This crate has some Cargo features which are disabled by default:
//! - **`derive`**: enables the custom derives for [`Atom`], [`AtomLogic`] and
//!   [`AtomInteger`]. It is disabled by default because it requires compiling
//!   a few dependencies for procedural macros.
//! - **`serde`**: enables the serde `Serialize` and `Deserialize` traits on
//!     `Atomic<T>` if `T` is serializable or deserializable.
//!

#![no_std]

#[cfg(test)]
#[macro_use]
extern crate std;

use core::fmt;
use crate::impls::{PrimitiveAtom, PrimitiveAtomLogic, PrimitiveAtomInteger};

pub mod impls;
#[cfg(test)]
mod tests;

/// Reexported from `std` for import convenience.
#[doc(no_inline)]
pub use core::sync::atomic::Ordering;

#[cfg(feature = "derive")]
pub use atomig_macro::{Atom, AtomInteger, AtomLogic};

// ===============================================================================================
// ===== User faced `Atom*` traits
// ===============================================================================================

/// Types that can be represented by a primitive type supporting atomic
/// operations.
///
/// This is trait is already implemented for all primitive types that support
/// atomic operations. It is also implemented for `f32`, `f64` and `char` as
/// all of those can be represented by a primitive atomic type. In addition to
/// this, you can implement this trait for your own types as long as they can
/// be represented as one such primitive type.
///
/// The `pack` and `unpack` methods define the conversion to and from the
/// atomic representation. The methods should be fairly fast, because they are
/// called frequently by [`Atomic`]: at least once for every method of
/// `Atomic`.
///
///
/// # Example
///
/// Imagine you have a `Port` type to represent a network port and use strong
/// typing. It is simply a newtype around a `u16`, so it is easily possible to
/// use atomic operations on it.
///
/// ```
/// use atomig::{Atom, Atomic, Ordering};
///
/// struct Port(u16);
///
/// impl Atom for Port {
///     type Repr = u16;
///     fn pack(self) -> Self::Repr {
///         self.0
///     }
///     fn unpack(src: Self::Repr) -> Self {
///         Port(src)
///     }
/// }
///
/// // Implementing `Atom` means that we can use `Atomic` with our type
/// let a = Atomic::new(Port(80));
/// a.store(Port(8080), Ordering::SeqCst);
/// ```
///
///
/// # Deriving this trait
///
/// Instead of implementing the trait manually (like shown above), you can
/// derive it automatically in many cases. In order to use that feature, you
/// have to enabled the Cargo feature 'derive'.
///
/// ```
/// use atomig::{Atom, Atomic, Ordering};
/// # #[cfg(feature = "derive")]
/// # fn main() {
///
/// #[derive(Atom)]
/// struct Port(u16);
///
/// let a = Atomic::new(Port(80));
/// a.store(Port(8080), Ordering::SeqCst);
/// # }
///
/// # #[cfg(not(feature = "derive"))]
/// # fn main() {}
/// ```
///
/// The trait can be automatically derived for two kinds of types:
/// - `struct` types with only *one* field. That field's type has to implement
///   `PrimitiveAtom` and is used as `Repr` type. Works with tuple structs or
///   normal structs with named fields.
/// - `enum` types that have a `#[repr(_)]` attribute specified and are C-like
///   (i.e. no variant has any fields). The primitive type specified in the
///   `#[repr(_)]` attribute is used as `Repr` type.
///
/// Example with enum:
///
/// ```
/// use atomig::{Atom, Atomic, Ordering};
/// # #[cfg(feature = "derive")]
/// # fn main() {
///
/// #[derive(Atom)]
/// #[repr(u8)]
/// enum Animal { Dog, Cat, Fox }
///
/// let a = Atomic::new(Animal::Cat);
/// a.store(Animal::Fox, Ordering::SeqCst);
/// # }
///
/// # #[cfg(not(feature = "derive"))]
/// # fn main() {}
/// ```
pub trait Atom {
    /// The atomic representation of this type.
    ///
    /// In this trait's implementations for the primitive types themselves,
    /// `Repr` is set to `Self`.
    type Repr: PrimitiveAtom;

    /// Converts the type to its atomic representation.
    fn pack(self) -> Self::Repr;

    /// Creates an instance of this type from the atomic representation.
    ///
    /// This method is usually only called with values that were returned by
    /// `pack`. So in theory, you can assume that the argument is a valid
    /// representation. *However*, there are two exceptions.
    ///
    /// If your type also implements `AtomLogic` or `AtomInteger`, results of
    /// those operations might get passed to `unpack`. Furthermore, this method
    /// can be called by anyone. So at the very least, you have to make sure
    /// that invalid input values do not lead to undefined behavior(e.g. memory
    /// unsafety)!
    fn unpack(src: Self::Repr) -> Self;
}

/// `Atom`s for which logical operations on their atomic representation make
/// sense.
///
/// Implementing this marker trait for your type makes it possible to use
/// [`Atomic::fetch_and`] and similar methods. Note that **the logical
/// operation is performed on the atomic representation of your type and _not_
/// on your type directly**!
///
/// Examples:
/// - Imagine you have a `Set(u64)` type which represents an integer set for
///   integers up to 63. The atomic representation is `u64` and the
///   `pack`/`unpack` methods are implemented as you would expect. In this
///   case, it makes sense to implement `AtomLogic` for `Set`: performing
///   bit-wise logical operations on the `u64` representation makes sense.
/// - Imagine you have `enum TriBool { Yes, Maybe, No }` which you represent by
///   `u8`. The `pack`/`unpack` methods use `No = 0`, `Maybe = 1` and `Yes = 2`
///   (or some other assignment). You also implement the logical operators from
///   `std::ops` for `TriBool`. In this case, it is probably *very wrong* to
///   implement `AtomLogic` for `TriBool`: the logical operations are performed
///   bit-wise on the `u8` which will result in very strange results (maybe
///   even in the value 3, which is not even valid). They will not use your
///   `std::ops` implementations!
///
///
/// # Deriving this trait
///
/// Like [`Atom`], this trait can automatically derived if the 'derive' Cargo
/// feature of this crate is enabled. This custom derive is simpler because
/// this is only a marker trait.
///
/// *However*, this trait cannot be derived for enums, as this is almost
/// certainly incorrect. While in C, enums basically list some constants and
/// often, these constants are used in bitwise logical operations, this is
/// often not valid in Rust. In Rust, a C-like enum can only have one of the
/// values listed in the definition and nothing else. Otherwise, UB is
/// triggered. Implementing this trait incorrectly does not cause UB, but the
/// `unpack` method will panic for unexpected values. If you still think you
/// want to implement this trait for your type, you have to do it manually.
pub trait AtomLogic: Atom
where
    Self::Repr: PrimitiveAtomLogic,
{}

/// `Atom`s for which integer operations on their atomic representation make
/// sense.
///
/// Implementing this marker trait for your type makes it possible to use
/// [`Atomic::fetch_add`] and similar methods. Note that **the integer
/// operation is performed on the atomic representation of your type and _not_
/// on your type directly**!
///
/// Examples:
/// - The `Set` and `TriBool` examples from the [`AtomLogic`] documentation
///   should both *not* implement `AtomInteger`, because addition on the
///   underlying integer does not result in any meaningful value for them.
/// - Imagine you have strong types for distance measurements, including
///   `Meter(u64)`. It makes sense to implement `AtomInteger` for that type,
///   because adding the representation (`u64`) makes sense.
/// - As another example for types that should *not* implement this type,
///   consider `f64`. It can be represented by `u64` and additions on `f64` do
///   make sense. *But* add adding the `u64` representations of two `f64` does
///   not yield any meaningful result!
///
///
/// # Deriving this trait
///
/// Like [`Atom`], this trait can automatically derived if the 'derive' Cargo
/// feature of this crate is enabled. This custom derive is simpler because
/// this is only a marker trait.
///
/// *However*, this trait cannot be derived for enums, as this is almost
/// certainly incorrect. While in C, enums basically list some constants and
/// often, these constants are added or subtracted from one another, this is
/// often not valid in Rust. In Rust, a C-like enum can only have one of the
/// values listed in the definition and nothing else. Otherwise, UB is
/// triggered. Implementing this trait incorrectly does not cause UB, but the
/// `unpack` method will panic for unexpected values. If you still think you
/// want to implement this trait for your type, you have to do it manually.
pub trait AtomInteger: Atom
where
    Self::Repr: PrimitiveAtomInteger,
{}



// ===============================================================================================
// ===== The `Atomic<T>` type
// ===============================================================================================

/// The main type of this library: a generic atomic type.
///
/// Via the methods of this type you can perform various atomic operations. You
/// can use any type `T` that implements [`Atom`]. This includes primitive
/// atomics (the ones found in `std::sync::atomic`), other primitive types and
/// potentially your own types. Additional methods are usable if `T` implements
/// [`AtomLogic`] or [`AtomInteger`].
///
/// All methods use [`Atom::pack`] and [`Atom::unpack`] to convert between the
/// underlying atomic value and the real value `T`. For types that implement
/// [`PrimitiveAtom`][impls::PrimitiveAtom], these two methods are a simple ID
/// function, meaning that there is no runtime overhead. Other types should
/// make sure their `pack` and `unpack` operations are fast, as they are used a
/// lot in this type.
///
/// For all methods that do a comparison (e.g. `compare_exchange`), keep in
/// mind that the comparison is performed on the bits of the underlying type
/// which can sometimes lead to unexpected behavior. For example, for floats,
/// there are many bit patterns that represent NaN. So the atomic might indeed
/// store a NaN representation at a moment, but `compare_exchange` called with
/// `current = NaN` might not swap, because both NaN differ in the bit
/// representation.
///
/// The interface of this type very closely matches the interface of the atomic
/// types in `std::sync::atomic`. The documentation was copied (and slightly
/// adjusted) from there.
pub struct Atomic<T: Atom>(<<T as Atom>::Repr as PrimitiveAtom>::Impl);

impl<T: Atom> Atomic<T> {
    /// Creates a new atomic value.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomig::Atomic;
    ///
    /// let x = Atomic::new(7u32);
    /// ```
    pub fn new(v: T) -> Self {
        Self(T::Repr::into_impl(v.pack()))
    }

    /// Consumes the atomic and returns the contained value.
    ///
    /// This is safe because passing `self` by value guarantees that no other
    /// threads are concurrently accessing the atomic data.
    pub fn into_inner(self) -> T {
        T::unpack(T::Repr::from_impl(self.0))
    }

    /// Loads the value from the atomic.
    ///
    /// `load` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation. Possible values are `SeqCst`, `Acquire` and
    /// `Relaxed`.
    ///
    /// # Panics
    ///
    /// Panics if `order` is `Release` or `AcqRel`.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let x = Atomic::new(5);
    /// assert_eq!(x.load(Ordering::SeqCst), 5);
    /// ```
    pub fn load(&self, order: Ordering) -> T {
        T::unpack(T::Repr::load(&self.0, order))
    }

    /// Stores a value into the atomic.
    ///
    /// `store` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation. Possible values are `SeqCst`, `Release` and
    /// `Relaxed`.
    ///
    /// # Panics
    ///
    /// Panics if `order` is `Acquire` or `AcqRel`.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let x = Atomic::new(5);
    ///
    /// x.store(10, Ordering::SeqCst);
    /// assert_eq!(x.load(Ordering::SeqCst), 10);
    /// ```
    pub fn store(&self, v: T, order: Ordering) {
        T::Repr::store(&self.0, v.pack(), order);
    }

    /// Stores a value into the atomic, returning the previous value.
    ///
    /// `swap` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation. All ordering modes are possible. Note that
    /// using `Acquire` makes the store part of this operation `Relaxed`, and
    /// using `Release` makes the load part `Relaxed`.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let x = Atomic::new(5);
    /// assert_eq!(x.swap(10, Ordering::SeqCst), 5);
    /// ```
    pub fn swap(&self, v: T, order: Ordering) -> T {
        T::unpack(T::Repr::swap(&self.0, v.pack(), order))
    }

    /// Stores a value into the atomic if the current value is the same as the
    /// `current` value.
    ///
    /// The return value is a result indicating whether the new value was
    /// written and containing the previous value. On success this value is
    /// guaranteed to be equal to `current`.
    ///
    /// `compare_exchange` takes two [`Ordering`] arguments to describe the
    /// memory ordering of this operation. The first describes the required
    /// ordering if the operation succeeds while the second describes the
    /// required ordering when the operation fails. Using `Acquire` as success
    /// ordering makes the store part of this operation `Relaxed`, and using
    /// `Release` makes the successful load `Relaxed`. The failure ordering can
    /// only be `SeqCst`, `Acquire` or `Relaxed` and must be equivalent to or
    /// weaker than the success ordering.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let x = Atomic::new(5);
    ///
    /// assert_eq!(
    ///     x.compare_exchange(5, 10, Ordering::Acquire, Ordering::Relaxed),
    ///     Ok(5),
    /// );
    /// assert_eq!(x.load(Ordering::Relaxed), 10);
    ///
    /// assert_eq!(
    ///     x.compare_exchange(6, 12, Ordering::SeqCst, Ordering::Acquire),
    ///     Err(10),
    /// );
    /// assert_eq!(x.load(Ordering::Relaxed), 10);
    /// ```
    pub fn compare_exchange(
        &self,
        current: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        T::Repr::compare_exchange(&self.0, current.pack(), new.pack(), success, failure)
            .map(T::unpack)
            .map_err(T::unpack)
    }

    /// Stores a value into the atomic if the current value is the same as the
    /// `current` value.
    ///
    /// Unlike `compare_exchange`, this function is allowed to spuriously fail
    /// even when the comparison succeeds, which can result in more efficient
    /// code on some platforms. The return value is a result indicating whether
    /// the new value was written and containing the previous value.
    ///
    /// `compare_exchange_weak` takes two [`Ordering`] arguments to describe
    /// the memory ordering of this operation. The first describes the required
    /// ordering if the operation succeeds while the second describes the
    /// required ordering when the operation fails. Using `Acquire` as success
    /// ordering makes the store part of this operation `Relaxed`, and using
    /// `Release` makes the successful load `Relaxed`. The failure ordering can
    /// only be `SeqCst`, `Acquire` or `Relaxed` and must be equivalent to or
    /// weaker than the success ordering.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let x = Atomic::new(4);
    ///
    /// let mut old = x.load(Ordering::Relaxed);
    /// loop {
    ///     let new = old * 2;
    ///     match x.compare_exchange_weak(old, new, Ordering::SeqCst, Ordering::Relaxed) {
    ///         Ok(_) => break,
    ///         Err(x) => old = x,
    ///     }
    /// }
    /// ```
    pub fn compare_exchange_weak(
        &self,
        current: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        T::Repr::compare_exchange_weak(&self.0, current.pack(), new.pack(), success, failure)
            .map(T::unpack)
            .map_err(T::unpack)
    }

    /// Fetches the value, and applies a function to it that returns an
    /// optional new value. Returns a `Result` of `Ok(previous_value)` if the
    /// function returned `Some(_)`, else `Err(previous_value)`.
    ///
    /// Note: This may call the function multiple times if the value has been
    /// changed from other threads in the meantime, as long as the function
    /// returns `Some(_)`, but the function will have been applied but once to
    /// the stored value.
    ///
    /// `fetch_update` takes two [`Ordering`] arguments to describe the memory
    /// ordering of this operation. The first describes the required ordering
    /// for loads and failed updates while the second describes the required
    /// ordering when the operation finally succeeds. Beware that this is
    /// different from the two modes in `compare_exchange`!
    ///
    /// Using `Acquire` as success ordering makes the store part of this
    /// operation `Relaxed`, and using `Release` makes the final successful
    /// load `Relaxed`. The (failed) load ordering can only be `SeqCst`,
    /// `Acquire` or `Relaxed` and must be equivalent to or weaker than the
    /// success ordering.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let x = Atomic::new(7);
    /// assert_eq!(x.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| None), Err(7));
    /// assert_eq!(x.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| Some(x + 1)), Ok(7));
    /// assert_eq!(x.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| Some(x + 1)), Ok(8));
    /// assert_eq!(x.load(Ordering::SeqCst), 9);
    /// ```
    pub fn fetch_update<F>(
        &self,
        set_order: Ordering,
        fetch_order: Ordering,
        mut f: F,
    ) -> Result<T, T>
    where
        F: FnMut(T) -> Option<T>
    {
        let f = |repr| f(T::unpack(repr)).map(Atom::pack);
        T::Repr::fetch_update(&self.0, set_order, fetch_order, f)
            .map(Atom::unpack)
            .map_err(Atom::unpack)
    }
}

// TODO: the `where` bound should not be necessary as the `AtomLogic` trait
// already specifies this. Maybe we can fix this in the future.
impl<T: AtomLogic> Atomic<T>
where
    T::Repr: PrimitiveAtomLogic,
{
    /// Bitwise "and" with the current value.
    ///
    /// Performs a bitwise "and" operation on the current value and the
    /// argument `val`, and sets the new value to the result.
    ///
    /// Returns the previous value.
    ///
    /// `fetch_and` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation. All ordering modes are possible. Note that
    /// using `Acquire` makes the store part of this operation `Relaxed`, and
    /// using `Release` makes the load part `Relaxed`.
    ///
    /// Examples
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let x = Atomic::new(0b101101);
    /// assert_eq!(x.fetch_and(0b110011, Ordering::SeqCst), 0b101101);
    /// assert_eq!(x.load(Ordering::SeqCst), 0b100001);
    /// ```
    pub fn fetch_and(&self, val: T, order: Ordering) -> T {
        T::unpack(T::Repr::fetch_and(&self.0, val.pack(), order))
    }

    /// Bitwise "nand" with the current value.
    ///
    /// Performs a bitwise "nand" operation on the current value and the
    /// argument `val`, and sets the new value to the result.
    ///
    /// Returns the previous value.
    ///
    /// `fetch_nand` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation. All ordering modes are possible. Note that
    /// using `Acquire` makes the store part of this operation `Relaxed`, and
    /// using `Release` makes the load part `Relaxed`.
    ///
    /// Examples
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let x = Atomic::new(0x13);
    /// assert_eq!(x.fetch_nand(0x31, Ordering::SeqCst), 0x13);
    /// assert_eq!(x.load(Ordering::SeqCst), !(0x13 & 0x31));
    /// ```
    pub fn fetch_nand(&self, val: T, order: Ordering) -> T {
        T::unpack(T::Repr::fetch_nand(&self.0, val.pack(), order))
    }

    /// Bitwise "or" with the current value.
    ///
    /// Performs a bitwise "or" operation on the current value and the
    /// argument `val`, and sets the new value to the result.
    ///
    /// Returns the previous value.
    ///
    /// `fetch_or` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation. All ordering modes are possible. Note that
    /// using `Acquire` makes the store part of this operation `Relaxed`, and
    /// using `Release` makes the load part `Relaxed`.
    ///
    /// Examples
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let x = Atomic::new(0b101101);
    /// assert_eq!(x.fetch_or(0b110011, Ordering::SeqCst), 0b101101);
    /// assert_eq!(x.load(Ordering::SeqCst), 0b111111);
    /// ```
    pub fn fetch_or(&self, val: T, order: Ordering) -> T {
        T::unpack(T::Repr::fetch_or(&self.0, val.pack(), order))
    }

    /// Bitwise "xor" with the current value.
    ///
    /// Performs a bitwise "xor" operation on the current value and the
    /// argument `val`, and sets the new value to the result.
    ///
    /// Returns the previous value.
    ///
    /// `fetch_xor` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation. All ordering modes are possible. Note that
    /// using `Acquire` makes the store part of this operation `Relaxed`, and
    /// using `Release` makes the load part `Relaxed`.
    ///
    /// Examples
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let x = Atomic::new(0b101101);
    /// assert_eq!(x.fetch_xor(0b110011, Ordering::SeqCst), 0b101101);
    /// assert_eq!(x.load(Ordering::SeqCst), 0b011110);
    /// ```
    pub fn fetch_xor(&self, val: T, order: Ordering) -> T {
        T::unpack(T::Repr::fetch_xor(&self.0, val.pack(), order))
    }
}


// TODO: the `where` bound should not be necessary as the `AtomInteger` trait
// already specifies this. Maybe we can fix this in the future.
impl<T: AtomInteger> Atomic<T>
where
    T::Repr: PrimitiveAtomInteger,
{
    /// Adds to the current value, returning the previous value.
    ///
    /// This operation wraps around on overflow.
    ///
    /// `fetch_add` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation. All ordering modes are possible. Note that
    /// using `Acquire` makes the store part of this operation `Relaxed`, and
    /// using `Release` makes the load part `Relaxed`.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let x = Atomic::new(0);
    /// assert_eq!(x.fetch_add(10, Ordering::SeqCst), 0);
    /// assert_eq!(x.load(Ordering::SeqCst), 10);
    /// ```
    pub fn fetch_add(&self, val: T, order: Ordering) -> T {
        T::unpack(T::Repr::fetch_add(&self.0, val.pack(), order))
    }

    /// Subtracts from the current value, returning the previous value.
    ///
    /// This operation wraps around on overflow.
    ///
    /// `fetch_sub` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation. All ordering modes are possible. Note that
    /// using `Acquire` makes the store part of this operation `Relaxed`, and
    /// using `Release` makes the load part `Relaxed`.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let x = Atomic::new(20);
    /// assert_eq!(x.fetch_sub(10, Ordering::SeqCst), 20);
    /// assert_eq!(x.load(Ordering::SeqCst), 10);
    /// ```
    pub fn fetch_sub(&self, val: T, order: Ordering) -> T {
        T::unpack(T::Repr::fetch_sub(&self.0, val.pack(), order))
    }

    /// Maximum with the current value.
    ///
    /// Finds the maximum of the current value and the argument `val`, and sets
    /// the new value to the result.
    ///
    /// Returns the previous value.
    ///
    /// `fetch_max` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation. All ordering modes are possible. Note that
    /// using `Acquire` makes the store part of this operation `Relaxed`, and
    /// using `Release` makes the load part `Relaxed`.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let foo = Atomic::new(23);
    /// assert_eq!(foo.fetch_max(42, Ordering::SeqCst), 23);
    /// assert_eq!(foo.load(Ordering::SeqCst), 42);
    /// ```
    ///
    /// If you want to obtain the maximum value in one step, you can use the
    /// following:
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let foo = Atomic::new(23);
    /// let bar = 42;
    /// let max_foo = foo.fetch_max(bar, Ordering::SeqCst).max(bar);
    /// assert!(max_foo == 42);
    /// ```
    pub fn fetch_max(&self, val: T, order: Ordering) -> T {
        T::unpack(T::Repr::fetch_max(&self.0, val.pack(), order))
    }

    /// Minimum with the current value.
    ///
    /// Finds the minimum of the current value and the argument `val`, and sets
    /// the new value to the result.
    ///
    /// Returns the previous value.
    ///
    /// `fetch_min` takes an [`Ordering`] argument which describes the memory
    /// ordering of this operation. All ordering modes are possible. Note that
    /// using `Acquire` makes the store part of this operation `Relaxed`, and
    /// using `Release` makes the load part `Relaxed`.
    ///
    /// # Examples
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let foo = Atomic::new(23);
    /// assert_eq!(foo.fetch_min(42, Ordering::Relaxed), 23);
    /// assert_eq!(foo.load(Ordering::Relaxed), 23);
    /// assert_eq!(foo.fetch_min(22, Ordering::Relaxed), 23);
    /// assert_eq!(foo.load(Ordering::Relaxed), 22);
    /// ```
    ///
    /// If you want to obtain the minimum value in one step, you can use the
    /// following:
    ///
    /// ```
    /// use atomig::{Atomic, Ordering};
    ///
    /// let foo = Atomic::new(23);
    /// let bar = 12;
    /// let min_foo = foo.fetch_min(bar, Ordering::SeqCst).min(bar);
    /// assert!(min_foo == 12);
    /// ```
    pub fn fetch_min(&self, val: T, order: Ordering) -> T {
        T::unpack(T::Repr::fetch_min(&self.0, val.pack(), order))
    }
}

impl<T: Atom + fmt::Debug> fmt::Debug for Atomic<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.load(Ordering::SeqCst).fmt(f)
    }
}

impl<T: Atom + Default> Default for Atomic<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Atom> From<T> for Atomic<T> {
    fn from(v: T) -> Self {
        Self::new(v)
    }
}

#[cfg(feature = "serde")]
impl<T: Atom + serde::Serialize> serde::Serialize for Atomic<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.load(Ordering::SeqCst).serialize(serializer)
    }
}


#[cfg(feature = "serde")]
impl<'de, T: Atom + serde::Deserialize<'de>> serde::Deserialize<'de> for Atomic<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer).map(Self::new)
    }
}
