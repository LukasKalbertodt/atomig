//! Traits for abstracting over `std` atomics. Basically implementation detail!
//!
//! This module only promises stability about the trait names and which types
//! these traits are implemented by (though, new impls can be added at any
//! time, of course). In particular, the traits' methods and other items are
//! not part of the public API of `atomig`. Those items are also hidden in the
//! documentation. And the traits are sealed anyway, so you can't implement
//! them for your own types.

use std::sync::atomic::{
    self, Ordering,
};
use super::{Atom, AtomLogic, AtomInteger};


// ===============================================================================================
// ===== All `Atomic*Impl` traits and `PrimitiveAtom`
// ===============================================================================================

mod sealed {
    /// You cannot implement this trait. That is the point.
    pub trait Sealed {}
}

/// Primitive types that can directly be used in an atomic way.
///
/// This trait is implemented exactly for every type that has a corresponding
/// atomic type in `std::sync::atomic`. You cannot implement this trait for
/// your own types; see [`Atom`] instead. This trait's items are not part of
/// the public API -- see the module docs.
pub trait PrimitiveAtom: Sized + Copy + sealed::Sealed {
    /// The standard library type that is the atomic version of `Self`.
    #[doc(hidden)]
    type Impl;

    #[doc(hidden)]
    fn into_impl(self) -> Self::Impl;
    #[doc(hidden)]
    fn from_impl(imp: Self::Impl) -> Self;

    #[doc(hidden)]
    fn get_mut(imp: &mut Self::Impl) -> &mut Self;
    #[doc(hidden)]
    fn load(imp: &Self::Impl, order: Ordering) -> Self;
    #[doc(hidden)]
    fn store(imp: &Self::Impl, v: Self, order: Ordering);

    #[doc(hidden)]
    fn swap(imp: &Self::Impl, v: Self, order: Ordering) -> Self;

    #[doc(hidden)]
    fn compare_exchange(
        imp: &Self::Impl,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self>;

    #[doc(hidden)]
    fn compare_exchange_weak(
        imp: &Self::Impl,
        current: Self,
        new: Self,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self>;

    #[doc(hidden)]
    fn fetch_update<F>(
        imp: &Self::Impl,
        set_order: Ordering,
        fetch_order: Ordering,
        f: F,
    ) -> Result<Self, Self>
    where
        F: FnMut(Self) -> Option<Self>;
}

/// Atomic types from `std::sync::atomic` which support logical operations.
///
/// You cannot implement this trait for your own types; see [`AtomLogic`]
/// instead. This trait's items are not part of the public API -- see the
/// module docs.
pub trait PrimitiveAtomLogic: PrimitiveAtom {
    #[doc(hidden)]
    fn fetch_and(imp: &Self::Impl, val: Self, order: Ordering) -> Self;
    #[doc(hidden)]
    fn fetch_nand(imp: &Self::Impl, val: Self, order: Ordering) -> Self;
    #[doc(hidden)]
    fn fetch_or(imp: &Self::Impl, val: Self, order: Ordering) -> Self;
    #[doc(hidden)]
    fn fetch_xor(imp: &Self::Impl, val: Self, order: Ordering) -> Self;
}

/// Atomic types from `std::sync::atomic` which support integer operations.
///
/// You cannot implement this trait for your own types; see [`AtomInteger`]
/// instead. This trait's items are not part of the public API -- see the
/// module docs.
pub trait PrimitiveAtomInteger: PrimitiveAtom {
    #[doc(hidden)]
    fn fetch_add(imp: &Self::Impl, val: Self, order: Ordering) -> Self;
    #[doc(hidden)]
    fn fetch_sub(imp: &Self::Impl, val: Self, order: Ordering) -> Self;

    #[doc(hidden)]
    fn fetch_max(imp: &Self::Impl, val: Self, order: Ordering) -> Self;
    #[doc(hidden)]
    fn fetch_min(imp: &Self::Impl, val: Self, order: Ordering) -> Self;
}



// ===============================================================================================
// ===== Implementations for standard library types
// ===============================================================================================

/// Expands to the `pack` and `unpack` methods implemented as ID function.
macro_rules! id_pack_unpack {
    () => {
        fn pack(self) -> Self::Repr {
            self
        }
        fn unpack(src: Self::Repr) -> Self {
            src
        }
    };
}

/// Expands to all methods from `AtomicImpl`, each forwarding to
/// `self.that_method`.
macro_rules! pass_through_methods {
    ($ty:ty) => {
        #[inline(always)]
        fn into_impl(self) -> Self::Impl {
            <$ty>::new(self)
        }

        #[inline(always)]
        fn from_impl(imp: Self::Impl) -> Self {
            imp.into_inner()
        }

        #[inline(always)]
        fn get_mut(imp: &mut Self::Impl) -> &mut Self {
            imp.get_mut()
        }

        #[inline(always)]
        fn load(imp: &Self::Impl, order: Ordering) -> Self {
            imp.load(order)
        }

        #[inline(always)]
        fn store(imp: &Self::Impl, v: Self, order: Ordering) {
            imp.store(v, order)
        }

        #[inline(always)]
        fn swap(imp: &Self::Impl, v: Self, order: Ordering) -> Self {
            imp.swap(v, order)
        }

        #[inline(always)]
        fn compare_exchange(
            imp: &Self::Impl,
            current: Self,
            new: Self,
            success: Ordering,
            failure: Ordering,
        ) -> Result<Self, Self> {
            imp.compare_exchange(current, new, success, failure)
        }

        #[inline(always)]
        fn compare_exchange_weak(
            imp: &Self::Impl,
            current: Self,
            new: Self,
            success: Ordering,
            failure: Ordering,
        ) -> Result<Self, Self> {
            imp.compare_exchange_weak(current, new, success, failure)
        }

        fn fetch_update<F>(
            imp: &Self::Impl,
            set_order: Ordering,
            fetch_order: Ordering,
            f: F,
        ) -> Result<Self, Self>
        where
            F: FnMut(Self) -> Option<Self>
        {
            imp.fetch_update(set_order, fetch_order, f)
        }
    };
}

/// Expands to all methods from `AtomicLogicImpl`, each forwarding to
/// `self.that_method`.
macro_rules! logical_pass_through_methods {
    () => {
        #[inline(always)]
        fn fetch_and(imp: &Self::Impl, val: Self, order: Ordering) -> Self {
            imp.fetch_and(val, order)
        }

        #[inline(always)]
        fn fetch_nand(imp: &Self::Impl, val: Self, order: Ordering) -> Self {
            imp.fetch_nand(val, order)
        }

        #[inline(always)]
        fn fetch_or(imp: &Self::Impl, val: Self, order: Ordering) -> Self {
            imp.fetch_or(val, order)
        }

        #[inline(always)]
        fn fetch_xor(imp: &Self::Impl, val: Self, order: Ordering) -> Self {
            imp.fetch_xor(val, order)
        }
    };
}

/// Expands to all methods from `AtomicIntegerImpl`, each forwarding to
/// `self.that_method`.
macro_rules! integer_pass_through_methods {
    () => {
        #[inline(always)]
        fn fetch_add(imp: &Self::Impl, val: Self, order: Ordering) -> Self {
            imp.fetch_add(val, order)
        }

        #[inline(always)]
        fn fetch_sub(imp: &Self::Impl, val: Self, order: Ordering) -> Self {
            imp.fetch_sub(val, order)
        }

        fn fetch_max(imp: &Self::Impl, val: Self, order: Ordering) -> Self {
            imp.fetch_max(val, order)
        }

        fn fetch_min(imp: &Self::Impl, val: Self, order: Ordering) -> Self {
            imp.fetch_min(val, order)
        }
    };
}

// ----- `*mut T` and `AtomicPtr` -----
impl<T> Atom for *mut T {
    type Repr = Self;
    id_pack_unpack!();
}

impl<T> sealed::Sealed for *mut T {}
impl<T> PrimitiveAtom for *mut T {
    type Impl = atomic::AtomicPtr<T>;
    pass_through_methods!(atomic::AtomicPtr<T>);
}



// ----- Integers and `bool` -----

macro_rules! impl_std_atomics {
    ($ty:ty, $impl_ty:ident, $is_int:ident) => {
        impl Atom for $ty {
            type Repr = Self;
            id_pack_unpack!();
        }

        impl sealed::Sealed for $ty {}
        impl AtomLogic for $ty {}
        impl PrimitiveAtom for $ty {
            type Impl = atomic::$impl_ty;
            pass_through_methods!(atomic::$impl_ty);
        }

        impl PrimitiveAtomLogic for $ty {
            logical_pass_through_methods!();
        }

        impl_std_atomics!(@int_methods $ty, $impl_ty, $is_int);
    };
    (@int_methods $ty:ty, $impl_ty:ident, true) => {
        impl AtomInteger for $ty {}
        impl PrimitiveAtomInteger for $ty {
            integer_pass_through_methods!();
        }
    };
    (@int_methods $ty:ty, $impl_ty:ident, false) => {};
}

impl_std_atomics!(bool, AtomicBool, false);
impl_std_atomics!(u8, AtomicU8, true);
impl_std_atomics!(i8, AtomicI8, true);
impl_std_atomics!(u16, AtomicU16, true);
impl_std_atomics!(i16, AtomicI16, true);
impl_std_atomics!(u32, AtomicU32, true);
impl_std_atomics!(i32, AtomicI32, true);
impl_std_atomics!(u64, AtomicU64, true);
impl_std_atomics!(i64, AtomicI64, true);
impl_std_atomics!(usize, AtomicUsize, true);
impl_std_atomics!(isize, AtomicIsize, true);

// ----- Implementations for non-atomic primitive types ------------------------------------------
impl Atom for f32 {
    type Repr = u32;
    fn pack(self) -> Self::Repr {
        self.to_bits()
    }
    fn unpack(src: Self::Repr) -> Self {
        Self::from_bits(src)
    }
}

impl Atom for f64 {
    type Repr = u64;
    fn pack(self) -> Self::Repr {
        self.to_bits()
    }
    fn unpack(src: Self::Repr) -> Self {
        Self::from_bits(src)
    }
}

impl Atom for char {
    type Repr = u32;
    fn pack(self) -> Self::Repr {
        self.into()
    }
    fn unpack(src: Self::Repr) -> Self {
        use std::convert::TryFrom;
        Self::try_from(src).expect("invalid value in <char as Atom>::unpack")
    }
}
