//! Traits for atomic implementations. You probably do not need to worry about
//! this module.

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
/// your own types; see [`Atom`] instead.
pub trait PrimitiveAtom: Sized + Copy + sealed::Sealed {
    /// The standard library type that is the atomic version of `Self`.
    type Impl: AtomicImpl<Inner = Self>;
}

/// Common interface of all atomic types in `std::sync::atomic`.
///
/// This trait is exactly implemented for all atomic types in
/// `std::sync::atomic` and you cannot and should not implement this trait for
/// your own types. Instead of using these methods directly, use
/// [`Atomic`][super::Atomic] which has the same interface.
pub trait AtomicImpl: Sized + sealed::Sealed {
    type Inner: PrimitiveAtom<Impl = Self>;

    fn new(v: Self::Inner) -> Self;
    fn get_mut(&mut self) -> &mut Self::Inner;
    fn into_inner(self) -> Self::Inner;
    fn load(&self, order: Ordering) -> Self::Inner;
    fn store(&self, v: Self::Inner, order: Ordering);

    fn swap(&self, v: Self::Inner, order: Ordering) -> Self::Inner;

    fn compare_and_swap(
        &self,
        current: Self::Inner,
        new: Self::Inner,
        order: Ordering,
    ) -> Self::Inner;

    fn compare_exchange(
        &self,
        current: Self::Inner,
        new: Self::Inner,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self::Inner, Self::Inner>;

    fn compare_exchange_weak(
        &self,
        current: Self::Inner,
        new: Self::Inner,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self::Inner, Self::Inner>;
}

/// Atomic types from `std::sync::atomic` which support logical operations.
pub trait AtomicLogicImpl: AtomicImpl {
    fn fetch_and(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
    fn fetch_nand(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
    fn fetch_or(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
    fn fetch_xor(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
}

/// Atomic types from `std::sync::atomic` which support integer operations.
pub trait AtomicIntegerImpl: AtomicImpl {
    fn fetch_add(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
    fn fetch_sub(&self, val: Self::Inner, order: Ordering) -> Self::Inner;

    fn fetch_max(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
    fn fetch_min(&self, val: Self::Inner, order: Ordering) -> Self::Inner;

    fn fetch_update<F>(
        &self,
        set_order: Ordering,
        fetch_order: Ordering,
        f: F,
    ) -> Result<Self::Inner, Self::Inner>
    where
        F: FnMut(Self::Inner) -> Option<Self::Inner>;
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
        fn new(v: Self::Inner) -> Self {
            <$ty>::new(v)
        }

        #[inline(always)]
        fn get_mut(&mut self) -> &mut Self::Inner {
            self.get_mut()
        }

        #[inline(always)]
        fn into_inner(self) -> Self::Inner {
            self.into_inner()
        }

        #[inline(always)]
        fn load(&self, order: Ordering) -> Self::Inner {
            self.load(order)
        }

        #[inline(always)]
        fn store(&self, v: Self::Inner, order: Ordering) {
            self.store(v, order)
        }

        #[inline(always)]
        fn swap(&self, v: Self::Inner, order: Ordering) -> Self::Inner {
            self.swap(v, order)
        }

        #[inline(always)]
        fn compare_and_swap(
            &self,
            current: Self::Inner,
            new: Self::Inner,
            order: Ordering,
        ) -> Self::Inner {
            self.compare_and_swap(current, new, order)
        }

        #[inline(always)]
        fn compare_exchange(
            &self,
            current: Self::Inner,
            new: Self::Inner,
            success: Ordering,
            failure: Ordering,
        ) -> Result<Self::Inner, Self::Inner> {
            self.compare_exchange(current, new, success, failure)
        }

        #[inline(always)]
        fn compare_exchange_weak(
            &self,
            current: Self::Inner,
            new: Self::Inner,
            success: Ordering,
            failure: Ordering,
        ) -> Result<Self::Inner, Self::Inner> {
            self.compare_exchange_weak(current, new, success, failure)
        }
    };
}

/// Expands to all methods from `AtomicLogicImpl`, each forwarding to
/// `self.that_method`.
macro_rules! logical_pass_through_methods {
    () => {
        #[inline(always)]
        fn fetch_and(&self, val: Self::Inner, order: Ordering) -> Self::Inner {
            self.fetch_and(val, order)
        }

        #[inline(always)]
        fn fetch_nand(&self, val: Self::Inner, order: Ordering) -> Self::Inner {
            self.fetch_nand(val, order)
        }

        #[inline(always)]
        fn fetch_or(&self, val: Self::Inner, order: Ordering) -> Self::Inner {
            self.fetch_or(val, order)
        }

        #[inline(always)]
        fn fetch_xor(&self, val: Self::Inner, order: Ordering) -> Self::Inner {
            self.fetch_xor(val, order)
        }
    };
}

/// Expands to all methods from `AtomicIntegerImpl`, each forwarding to
/// `self.that_method`.
macro_rules! integer_pass_through_methods {
    () => {
        #[inline(always)]
        fn fetch_add(&self, val: Self::Inner, order: Ordering) -> Self::Inner {
            self.fetch_add(val, order)
        }

        #[inline(always)]
        fn fetch_sub(&self, val: Self::Inner, order: Ordering) -> Self::Inner {
            self.fetch_sub(val, order)
        }

        fn fetch_max(&self, val: Self::Inner, order: Ordering) -> Self::Inner {
            self.fetch_max(val, order)
        }

        fn fetch_min(&self, val: Self::Inner, order: Ordering) -> Self::Inner {
            self.fetch_min(val, order)
        }

        fn fetch_update<F>(
            &self,
            set_order: Ordering,
            fetch_order: Ordering,
            f: F,
        ) -> Result<Self::Inner, Self::Inner>
        where
            F: FnMut(Self::Inner) -> Option<Self::Inner>
        {
            self.fetch_update(set_order, fetch_order, f)
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
}

impl<T> sealed::Sealed for atomic::AtomicPtr<T> {}
impl<T> AtomicImpl for atomic::AtomicPtr<T> {
    type Inner = *mut T;
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
        impl PrimitiveAtom for $ty {
            type Impl = atomic::$impl_ty;
        }

        impl AtomLogic for $ty {}

        impl sealed::Sealed for atomic::$impl_ty {}
        impl AtomicImpl for atomic::$impl_ty {
            type Inner = $ty;
            pass_through_methods!(atomic::$impl_ty);
        }

        impl AtomicLogicImpl for atomic::$impl_ty {
            logical_pass_through_methods!();
        }

        impl_std_atomics!(@int_methods $ty, $impl_ty, $is_int);
    };
    (@int_methods $ty:ty, $impl_ty:ident, true) => {
        impl AtomInteger for $ty {}

        impl AtomicIntegerImpl for atomic::$impl_ty {
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
