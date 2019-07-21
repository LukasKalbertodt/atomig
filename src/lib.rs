#![feature(cfg_target_has_atomic)]

use std::sync::atomic::{
    self, Ordering,
};



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



// ===============================================================================================
// ===== All `Atomic*Impl` traits and `PrimitiveAtom`
// ===============================================================================================

mod sealed {
    /// You cannot implement this trait. That is the point.
    pub trait Sealed {}
}

/// Primitive types that can directly be used in an atomic way. You probably do
/// not need to worry about this trait.
///
/// This trait is implemented exactly for every type that has a corresponding
/// atomic type in `std::sync::atomic`. You cannot implement this trait for
/// your own types; see [`Atom`] instead.
pub trait PrimitiveAtom: Sized + Copy + sealed::Sealed {
    /// The standard library type that is the atomic version of `Self`.
    type Impl: AtomicImpl<Inner = Self>;
}

/// Common interface of all atomic types in `std::sync::atomic`. You probably
/// do not need to worry about this trait.
///
/// This trait is exactly implemented for all atomic types in
/// `std::sync::atomic` and you cannot and should not implement this trait for
/// your own types. Instead of using these methods directly, use [`Atomic`]
/// which has the same interface.
pub trait AtomicImpl: Sized + sealed::Sealed {
    type Inner: PrimitiveAtom<Impl = Self>;

    fn new(v: Self::Inner) -> Self;
    fn get_mut(&mut self) -> &mut Self::Inner;
    fn into_inner(self) -> Self::Inner;
    fn load(&self, order: Ordering) -> Self::Inner;
    fn store(&self, v: Self::Inner, order: Ordering);

    #[cfg(target_has_atomic = "cas")]
    fn swap(&self, v: Self::Inner, order: Ordering) -> Self::Inner;

    #[cfg(target_has_atomic = "cas")]
    fn compare_and_swap(
        &self,
        current: Self::Inner,
        new: Self::Inner,
        order: Ordering,
    ) -> Self::Inner;

    #[cfg(target_has_atomic = "cas")]
    fn compare_exchange(
        &self,
        current: Self::Inner,
        new: Self::Inner,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self::Inner, Self::Inner>;

    #[cfg(target_has_atomic = "cas")]
    fn compare_exchange_weak(
        &self,
        current: Self::Inner,
        new: Self::Inner,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self::Inner, Self::Inner>;
}

/// Atomic types from `std::sync::atomic` which support logical operations. You
/// probably do not need to worry about this trait.
#[cfg(target_has_atomic = "cas")]
pub trait AtomicLogicImpl: AtomicImpl {
    fn fetch_and(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
    fn fetch_nand(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
    fn fetch_or(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
    fn fetch_xor(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
}

/// Atomic types from `std::sync::atomic` which support integer operations. You
/// probably do not need to worry about this trait.
#[cfg(target_has_atomic = "cas")]
pub trait AtomicIntegerImpl: AtomicImpl {
    fn fetch_add(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
    fn fetch_sub(&self, val: Self::Inner, order: Ordering) -> Self::Inner;
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
        #[cfg(target_has_atomic = "cas")]
        fn swap(&self, v: Self::Inner, order: Ordering) -> Self::Inner {
            self.swap(v, order)
        }

        #[inline(always)]
        #[cfg(target_has_atomic = "cas")]
        fn compare_and_swap(
            &self,
            current: Self::Inner,
            new: Self::Inner,
            order: Ordering,
        ) -> Self::Inner {
            self.compare_and_swap(current, new, order)
        }

        #[inline(always)]
        #[cfg(target_has_atomic = "cas")]
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
        #[cfg(target_has_atomic = "cas")]
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
    };
}

// ----- `*mut T` and `AtomicPtr` -----
#[cfg(target_has_atomic = "ptr")]
impl<T> Atom for *mut T {
    type Repr = Self;
    id_pack_unpack!();
}

#[cfg(target_has_atomic = "ptr")]
impl<T> sealed::Sealed for *mut T {}
#[cfg(target_has_atomic = "ptr")]
impl<T> PrimitiveAtom for *mut T {
    type Impl = atomic::AtomicPtr<T>;
}

#[cfg(target_has_atomic = "ptr")]
impl<T> sealed::Sealed for atomic::AtomicPtr<T> {}
#[cfg(target_has_atomic = "ptr")]
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

        #[cfg(target_has_atomic = "cas")]
        impl AtomicLogicImpl for atomic::$impl_ty {
            logical_pass_through_methods!();
        }

        #[cfg(target_has_atomic = "cas")]
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

#[cfg(target_has_atomic = "8")] impl_std_atomics!(bool, AtomicBool, false);
#[cfg(target_has_atomic = "8")] impl_std_atomics!(u8, AtomicU8, true);
#[cfg(target_has_atomic = "8")] impl_std_atomics!(i8, AtomicI8, true);
#[cfg(target_has_atomic = "16")] impl_std_atomics!(u16, AtomicU16, true);
#[cfg(target_has_atomic = "16")] impl_std_atomics!(i16, AtomicI16, true);
#[cfg(target_has_atomic = "32")] impl_std_atomics!(u32, AtomicU32, true);
#[cfg(target_has_atomic = "32")] impl_std_atomics!(i32, AtomicI32, true);
#[cfg(target_has_atomic = "64")] impl_std_atomics!(u64, AtomicU64, true);
#[cfg(target_has_atomic = "64")] impl_std_atomics!(i64, AtomicI64, true);
#[cfg(target_has_atomic = "ptr")] impl_std_atomics!(usize, AtomicUsize, true);
#[cfg(target_has_atomic = "ptr")] impl_std_atomics!(isize, AtomicIsize, true);



// ===============================================================================================
// ===== Utilities
// ===============================================================================================

/// Tiny type alias for avoid long paths in this codebase.
type Impl<A> = <<A as Atom>::Repr as PrimitiveAtom>::Impl;
