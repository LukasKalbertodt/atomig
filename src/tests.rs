use crate::{Atom, Atomic, Ordering};

// ===============================================================================================
// ===== Single test snippets
// ===============================================================================================

macro_rules! generic_tests {
    ($ty:ty, $val0:expr, $val1:expr) => {
        #[test]
        fn new() {
            let _: Atomic<$ty> = Atomic::new($val0);
        }

        #[test]
        fn load_store() {
            let a = Atomic::new($val0);
            assert_eq!(a.load(Ordering::SeqCst), $val0);

            a.store($val1, Ordering::SeqCst);
            assert_eq!(a.load(Ordering::SeqCst), $val1);
        }

        #[test]
        fn into_inner() {
            let a = Atomic::new($val0);
            assert_eq!(a.into_inner(), $val0);
        }

        #[test]
        fn swap() {
            let a = Atomic::new($val0);
            assert_eq!(a.swap($val1, Ordering::SeqCst), $val0);
            assert_eq!(a.load(Ordering::SeqCst), $val1);
        }

        #[test]
        fn fmt_debug() {
            let a = Atomic::new($val0);
            assert_eq!(format!("{:?}", a), format!("{:?}", $val0));
        }

        #[test]
        fn from() {
            let a = Atomic::new($val0);
            let b: Atomic<$ty> = $val0.into();
            assert_eq!(a.load(Ordering::SeqCst), b.load(Ordering::SeqCst));
        }

        // TODO: compare_and_* methods
    };
}

macro_rules! default_tests {
    ($ty:ty) => {
        #[test]
        fn default() {
            let a: Atomic<$ty> = Default::default();
            assert_eq!(a.load(Ordering::SeqCst), <$ty>::default());
        }
    };
}

macro_rules! serde_tests {
    ($ty:ty, $val0:expr) => {
        #[cfg(feature = "serde")]
        use bincode;

        #[cfg(feature = "serde")]
        #[test]
        fn test_serde_round_trip() {
            let src = Atomic::new($val0);
            let serialized = bincode::serialize(&src).unwrap();
            let deserialized: Atomic<$ty> = bincode::deserialize(&serialized).unwrap();
            assert_eq!(src.load(Ordering::SeqCst), deserialized.load(Ordering::SeqCst));
        }
    };
}

macro_rules! logic_tests {
    ($val0:expr, $val1:expr) => {
        #[test]
        fn logic() {
            let a = Atomic::new($val0);
            assert_eq!(a.fetch_and($val1, Ordering::SeqCst), $val0);
            assert_eq!(a.load(Ordering::SeqCst), $val0 & $val1);

            let a = Atomic::new($val0);
            assert_eq!(a.fetch_nand($val1, Ordering::SeqCst), $val0);
            assert_eq!(a.load(Ordering::SeqCst), !($val0 & $val1));

            let a = Atomic::new($val0);
            assert_eq!(a.fetch_or($val1, Ordering::SeqCst), $val0);
            assert_eq!(a.load(Ordering::SeqCst), $val0 | $val1);

            let a = Atomic::new($val0);
            assert_eq!(a.fetch_xor($val1, Ordering::SeqCst), $val0);
            assert_eq!(a.load(Ordering::SeqCst), $val0 ^ $val1);
        }
    };
}

macro_rules! int_tests {
    ($val0:expr, $val1:expr) => {
        #[test]
        fn integer() {
            let a = Atomic::new($val0);
            assert_eq!(a.fetch_add($val1, Ordering::SeqCst), $val0);
            assert_eq!(a.load(Ordering::SeqCst), $val0.wrapping_add($val1));

            let a = Atomic::new($val0);
            assert_eq!(a.fetch_sub($val1, Ordering::SeqCst), $val0);
            assert_eq!(a.load(Ordering::SeqCst), $val0.wrapping_sub($val1));
        }
    };
}

/// If the first token is `y`, emit the following tokens (inside a brace),
/// otherwise emit nothing.
macro_rules! emit_if {
    (n, { $($t:tt)* }) => {};
    (y, { $($t:tt)* }) => { $($t)* };
}


// ===============================================================================================
// ===== Actual tests of different types
// ===============================================================================================

macro_rules! gen_tests_for_primitives {
    (
        $mod_name:ident, $ty:ty, $val0:expr, $val1:expr,
        [$with_logic:ident $with_int:ident]
    ) => {
        mod $mod_name {
            use super::*;

            generic_tests!($ty, $val0, $val1);
            default_tests!($ty);
            serde_tests!($ty, $val0);

            emit_if!($with_logic, { logic_tests!($val0, $val1); });
            emit_if!($with_int, { int_tests!($val0, $val1); });
        }
    };
}

//                        mod     ty     val0    val1     [logic int]
gen_tests_for_primitives!(_bool,  bool,  true,   false,   [y n]);
gen_tests_for_primitives!(_u8,    u8,    7u8,    33u8,    [y y]);
gen_tests_for_primitives!(_i8,    i8,    7i8,    33i8,    [y y]);
gen_tests_for_primitives!(_u16,   u16,   7u16,   33u16,   [y y]);
gen_tests_for_primitives!(_i16,   i16,   7i16,   33i16,   [y y]);
gen_tests_for_primitives!(_u32,   u32,   7u32,   33u32,   [y y]);
gen_tests_for_primitives!(_i32,   i32,   7i32,   33i32,   [y y]);
gen_tests_for_primitives!(_u64,   u64,   7u64,   33u64,   [y y]);
gen_tests_for_primitives!(_i64,   i64,   7i64,   33i64,   [y y]);
gen_tests_for_primitives!(_usize, usize, 7usize, 33usize, [y y]);
gen_tests_for_primitives!(_isize, isize, 7isize, 33isize, [y y]);
gen_tests_for_primitives!(_f32,   f32,   7.0f32, 33.0f32, [n n]);
gen_tests_for_primitives!(_f64,   f64,   7.0f64, 33.0f64, [n n]);
gen_tests_for_primitives!(_char,  char,  'x',    '♥',     [n n]);

mod _ptr {
    use super::*;
    generic_tests!(Foo, Foo::Nothing, Foo::Set(0b101));
}

mod custom {
    use super::*;
    generic_tests!(Foo, Foo::Nothing, Foo::Set(0b101));
    default_tests!(Foo);
}


#[derive(Debug, PartialEq, Eq)]
enum Foo {
    Nothing,
    Set(u8),
}

impl Default for Foo {
    fn default() -> Self {
        Foo::Set(0b10101010)
    }
}

impl Atom for Foo {
    type Repr = u16;
    fn pack(self) -> Self::Repr {
        match self {
            Foo::Nothing => 0x01FF,
            Foo::Set(s) => s as u16,
        }
    }
    fn unpack(src: Self::Repr) -> Self {
        if src & 0x0100 != 0 {
            Foo::Nothing
        } else {
            Foo::Set((src & 0xFF) as u8)
        }
    }
}

macro_rules! gen_tests_for_opt_non_zeroes {
    ($mod_name:ident, $ty:ident) => {
        mod $mod_name {
            use super::*;
            use std::num::$ty;

            generic_tests!(Option<$ty>, $ty::new(7), $ty::new(33));
            default_tests!(Option<$ty>);
            serde_tests!(Option<$ty>, $ty::new(7));

            #[test]
            fn integer() {
                let a = Atomic::new($ty::new(7));
                assert_eq!(a.fetch_add($ty::new(33), Ordering::SeqCst), $ty::new(7));
                assert_eq!(a.load(Ordering::SeqCst), $ty::new(40));

                let a = Atomic::new($ty::new(33));
                assert_eq!(a.fetch_sub($ty::new(7), Ordering::SeqCst), $ty::new(33));
                assert_eq!(a.load(Ordering::SeqCst), $ty::new(26));

                let a = Atomic::new($ty::new(33));
                assert_eq!(a.fetch_sub(None, Ordering::SeqCst), $ty::new(33));
                assert_eq!(a.load(Ordering::SeqCst), $ty::new(33));

                let a = Atomic::new($ty::new(27));
                assert_eq!(a.fetch_sub($ty::new(27), Ordering::SeqCst), $ty::new(27));
                assert_eq!(a.load(Ordering::SeqCst), None);
            }
        }
    };
}

gen_tests_for_opt_non_zeroes!(nz_u8,    NonZeroU8);
gen_tests_for_opt_non_zeroes!(nz_i8,    NonZeroI8);
gen_tests_for_opt_non_zeroes!(nz_u16,   NonZeroU16);
gen_tests_for_opt_non_zeroes!(nz_i16,   NonZeroI16);
gen_tests_for_opt_non_zeroes!(nz_u32,   NonZeroU32);
gen_tests_for_opt_non_zeroes!(nz_i32,   NonZeroI32);
gen_tests_for_opt_non_zeroes!(nz_u64,   NonZeroU64);
gen_tests_for_opt_non_zeroes!(nz_i64,   NonZeroI64);
gen_tests_for_opt_non_zeroes!(nz_usize, NonZeroUsize);
gen_tests_for_opt_non_zeroes!(nz_isize, NonZeroIsize);
