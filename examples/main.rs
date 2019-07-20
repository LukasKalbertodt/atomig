use atomig::{Atom, Atomic, AtomicImpl};

fn main() {
    let mut v = 0u32;
    let a = Atomic::new(&mut v as *mut _);
    let b = Atomic::new(Age(3));
}

struct Age(u8);

impl Atom for Age {
    type Impl = std::sync::atomic::AtomicU8;

    fn pack(self) -> <Self::Impl as AtomicImpl>::Inner {
        self.0
    }
    fn unpack(src: <Self::Impl as AtomicImpl>::Inner) -> Self {
        Self(src)
    }
}
