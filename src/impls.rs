//! Traits for abstracting over `std` atomics. Basically implementation detail!
//!
//! This module only promises stability about the trait names and which types
//! these traits are implemented by (though, new impls can be added at any
//! time, of course). In particular, the traits' methods and other items are
//! not part of the public API of `atomig`. Those items are also hidden in the
//! documentation. And the traits are sealed anyway, so you can't implement
//! them for your own types.

use core::{num::Wrapping, sync::atomic::{self, Ordering}};
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
    ($ty:ty, $non_zero_ty:ident, $impl_ty:ident, $is_int:ident) => {
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

        impl_std_atomics!(@int_methods $ty, $non_zero_ty, $impl_ty, $is_int);
    };
    (@int_methods $ty:ty, $non_zero_ty:ident, $impl_ty:ident, true) => {
        impl AtomInteger for $ty {}
        impl PrimitiveAtomInteger for $ty {
            integer_pass_through_methods!();
        }

        impl Atom for core::num::$non_zero_ty {
            type Repr = $ty;
            fn pack(self) -> Self::Repr {
                self.get()
            }

            fn unpack(src: Self::Repr) -> Self {
                // Since `AtomLogic` and `AtomInteger` is not implemented for
                // NonZero types, there is no way zero can be the result of any
                // atomic operation. Thus this should never happen.
                Self::new(src).expect("zero value in `Atom::unpack` for NonZero type")
            }
        }
    };
    (@int_methods $ty:ty, $non_zero_ty:ident, $impl_ty:ident, false) => {};
}

impl_std_atomics!(bool, _Dummy, AtomicBool, false);
impl_std_atomics!(u8, NonZeroU8, AtomicU8, true);
impl_std_atomics!(i8, NonZeroI8, AtomicI8, true);
impl_std_atomics!(u16, NonZeroU16, AtomicU16, true);
impl_std_atomics!(i16, NonZeroI16, AtomicI16, true);
impl_std_atomics!(u32, NonZeroU32, AtomicU32, true);
impl_std_atomics!(i32, NonZeroI32, AtomicI32, true);
impl_std_atomics!(u64, NonZeroU64, AtomicU64, true);
impl_std_atomics!(i64, NonZeroI64, AtomicI64, true);
impl_std_atomics!(usize, NonZeroUsize, AtomicUsize, true);
impl_std_atomics!(isize, NonZeroIsize, AtomicIsize, true);

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
        use core::convert::TryFrom;
        Self::try_from(src).expect("invalid value in <char as Atom>::unpack")
    }
}

// We do not implement `AtomInteger` as, to me, it seems like the exact adding
// and subtraction behavior of integer atomics is not defined anywhere.
impl<T: Atom> Atom for Wrapping<T> {
    type Repr = T::Repr;
    fn pack(self) -> Self::Repr {
        self.0.pack()
    }
    fn unpack(src: Self::Repr) -> Self {
        Self(T::unpack(src))
    }
}
impl<T: AtomLogic> AtomLogic for Wrapping<T> where T::Repr: PrimitiveAtomLogic {}


impl<T> Atom for core::ptr::NonNull<T> {
    type Repr = *mut T;
    fn pack(self) -> Self::Repr {
        self.as_ptr().pack()
    }
    fn unpack(src: Self::Repr) -> Self {
        Self::new(<*mut T>::unpack(src))
            .expect("null value in `<NonNull<T> as Atom>::unpack`")
    }
}

impl<T> Atom for Option<core::ptr::NonNull<T>> {
    type Repr = *mut T;
    fn pack(self) -> Self::Repr {
        self.map(|nn| nn.as_ptr())
            .unwrap_or(core::ptr::null_mut())
            .pack()
    }
    fn unpack(src: Self::Repr) -> Self {
        if src.is_null() {
            None
        } else {
            Some(core::ptr::NonNull::new(<*mut T>::unpack(src)).unwrap())
        }
    }
}

macro_rules! impl_option_non_zero {
    ($ty:ident = $repr:ty) => {
        impl Atom for Option<core::num::$ty> {
            type Repr = $repr;
            fn pack(self) -> Self::Repr {
                self.map(core::num::$ty::get).unwrap_or(0).pack()
            }
            fn unpack(src: Self::Repr) -> Self {
                core::num::$ty::new(src)
            }
        }

        // Semantically, an `Option<NonZeroFoo>` represents `Foo` exactly. It
        // also has the exact same memory layout. It's just that we assign
        // the "symbol" `None` to 0. Any integer operation that leads to 0 on
        // the underlying type will result in `None`.
        impl AtomInteger for Option<core::num::$ty> {}
    };
}

impl_option_non_zero!(NonZeroU8 = u8);
impl_option_non_zero!(NonZeroI8 = i8);
impl_option_non_zero!(NonZeroU16 = u16);
impl_option_non_zero!(NonZeroI16 = i16);
impl_option_non_zero!(NonZeroU32 = u32);
impl_option_non_zero!(NonZeroI32 = i32);
impl_option_non_zero!(NonZeroU64 = u64);
impl_option_non_zero!(NonZeroI64 = i64);
impl_option_non_zero!(NonZeroUsize = usize);
impl_option_non_zero!(NonZeroIsize = isize);

/// This is just a dummy module to have doc tests.
///
/// ```
/// use atomig::{Atom, AtomLogic, AtomInteger, impls::{PrimitiveAtomLogic, PrimitiveAtomInteger}};
///
/// fn assert_impl_atom<T: Atom>() {}
/// fn assert_impl_atom_logic<T: AtomLogic>()
/// where
///     T::Repr: PrimitiveAtomLogic,
/// {}
/// fn assert_impl_atom_all<T: AtomLogic + AtomInteger>()
/// where
///     T::Repr: PrimitiveAtomInteger + PrimitiveAtomLogic,
/// {}
///
/// assert_impl_atom_all::<u8>();
/// assert_impl_atom_all::<i8>();
/// assert_impl_atom_all::<u16>();
/// assert_impl_atom_all::<i16>();
/// assert_impl_atom_all::<u32>();
/// assert_impl_atom_all::<i32>();
/// assert_impl_atom_all::<u64>();
/// assert_impl_atom_all::<i64>();
/// assert_impl_atom_all::<usize>();
/// assert_impl_atom_all::<isize>();
///
/// assert_impl_atom_logic::<bool>();
///
/// assert_impl_atom::<*mut ()>();
/// assert_impl_atom::<*mut String>();
/// assert_impl_atom::<core::ptr::NonNull<()>>();
/// assert_impl_atom::<core::ptr::NonNull<String>>();
/// assert_impl_atom::<Option<core::ptr::NonNull<()>>>();
/// assert_impl_atom::<Option<core::ptr::NonNull<String>>>();
///
/// assert_impl_atom::<char>();
/// assert_impl_atom::<f32>();
/// assert_impl_atom::<f64>();
///
/// assert_impl_atom::<core::num::NonZeroU8>();
/// assert_impl_atom::<core::num::NonZeroI8>();
/// assert_impl_atom::<core::num::NonZeroU16>();
/// assert_impl_atom::<core::num::NonZeroI16>();
/// assert_impl_atom::<core::num::NonZeroU32>();
/// assert_impl_atom::<core::num::NonZeroI32>();
/// assert_impl_atom::<core::num::NonZeroU64>();
/// assert_impl_atom::<core::num::NonZeroI64>();
/// assert_impl_atom::<core::num::NonZeroUsize>();
/// assert_impl_atom::<core::num::NonZeroIsize>();
///
/// assert_impl_atom_logic::<core::num::Wrapping<u8>>();
/// assert_impl_atom_logic::<core::num::Wrapping<i8>>();
/// assert_impl_atom_logic::<core::num::Wrapping<u16>>();
/// assert_impl_atom_logic::<core::num::Wrapping<i16>>();
/// assert_impl_atom_logic::<core::num::Wrapping<u32>>();
/// assert_impl_atom_logic::<core::num::Wrapping<i32>>();
/// assert_impl_atom_logic::<core::num::Wrapping<u64>>();
/// assert_impl_atom_logic::<core::num::Wrapping<i64>>();
/// assert_impl_atom_logic::<core::num::Wrapping<usize>>();
/// assert_impl_atom_logic::<core::num::Wrapping<isize>>();
/// ```
mod tests {}
