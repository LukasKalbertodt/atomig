#![feature(cfg_target_has_atomic)]

use std::fmt;
use crate::impls::{PrimitiveAtom, AtomicImpl, AtomicLogicImpl, AtomicIntegerImpl};

pub mod impls;
#[cfg(test)]
mod tests;

/// Reexported from `std` for import convenience.
#[doc(no_inline)]
pub use std::sync::atomic::Ordering;

// ===============================================================================================
// ===== User faced `Atom*` traits
// ===============================================================================================

/// Types that can be represented by a primitive type supporting atomic
/// operations.
///
/// This is trait is already implemented for all primitive types that support
/// atomic operations. But in addition to this, you can implement this trait
/// for your own types as long as they can be represented as one such primitive
/// type.
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
/// use std::sync::atomic::Ordering;
/// use atomig::{Atom, Atomic};
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
/// // Implementing `Atom` means that we can use `Atomic` with out type
/// let a = Atomic::new(Port(80));
/// a.store(Port(8080), Ordering::SeqCst);
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
    /// that invalid input values do not lead to memory unsafety!
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
pub trait AtomLogic: Atom
where
    Impl<Self>: AtomicLogicImpl
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
pub trait AtomInteger: Atom
where
    Impl<Self>: AtomicIntegerImpl,
{}



// ===============================================================================================
// ===== The `Atomic<T>` type
// ===============================================================================================

pub struct Atomic<T: Atom>(Impl<T>);

impl<T: Atom> Atomic<T> {
    pub fn new(v: T) -> Self {
        Self(Impl::<T>::new(v.pack()))
    }

    // fn get_mut(&mut self) -> &mut Self::Inner;

    pub fn into_inner(self) -> T {
        T::unpack(self.0.into_inner())
    }
    pub fn load(&self, order: Ordering) -> T {
        T::unpack(self.0.load(order))
    }
    pub fn store(&self, v: T, order: Ordering) {
        self.0.store(v.pack(), order);
    }

    #[cfg(target_has_atomic = "cas")]
    pub fn swap(&self, v: T, order: Ordering) -> T {
        T::unpack(self.0.swap(v.pack(), order))
    }

    #[cfg(target_has_atomic = "cas")]
    pub fn compare_and_swap(&self, current: T, new: T, order: Ordering) -> T {
        T::unpack(self.0.compare_and_swap(current.pack(), new.pack(), order))
    }

    #[cfg(target_has_atomic = "cas")]
    pub fn compare_exchange(
        &self,
        current: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        self.0.compare_exchange(current.pack(), new.pack(), success, failure)
            .map(T::unpack)
            .map_err(T::unpack)
    }

    #[cfg(target_has_atomic = "cas")]
    pub fn compare_exchange_weak(
        &self,
        current: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        self.0.compare_exchange_weak(current.pack(), new.pack(), success, failure)
            .map(T::unpack)
            .map_err(T::unpack)
    }
}

// TODO: the `where` bound should not be necessary as the `AtomLogic` trait
// already specifies this. Maybe we can fix this in the future.
#[cfg(target_has_atomic = "cas")]
impl<T: AtomLogic> Atomic<T>
where
    Impl<T>: AtomicLogicImpl,
{
    pub fn fetch_and(&self, val: T, order: Ordering) -> T {
        T::unpack(self.0.fetch_and(val.pack(), order))
    }
    pub fn fetch_nand(&self, val: T, order: Ordering) -> T {
        T::unpack(self.0.fetch_nand(val.pack(), order))
    }
    pub fn fetch_or(&self, val: T, order: Ordering) -> T {
        T::unpack(self.0.fetch_or(val.pack(), order))
    }
    pub fn fetch_xor(&self, val: T, order: Ordering) -> T {
        T::unpack(self.0.fetch_xor(val.pack(), order))
    }
}


// TODO: the `where` bound should not be necessary as the `AtomInteger` trait
// already specifies this. Maybe we can fix this in the future.
#[cfg(target_has_atomic = "cas")]
impl<T: AtomInteger> Atomic<T>
where
    Impl<T>: AtomicIntegerImpl,
{
    pub fn fetch_add(&self, val: T, order: Ordering) -> T {
        T::unpack(self.0.fetch_add(val.pack(), order))
    }
    pub fn fetch_sub(&self, val: T, order: Ordering) -> T {
        T::unpack(self.0.fetch_sub(val.pack(), order))
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

// ===============================================================================================
// ===== Utilities
// ===============================================================================================

/// Tiny type alias for avoid long paths in this codebase.
type Impl<A> = <<A as Atom>::Repr as PrimitiveAtom>::Impl;
