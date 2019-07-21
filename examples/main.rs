use std::sync::atomic::Ordering;
use atomig::{Atom, Atomic};

fn main() {
    let mut v = 0u32;
    let a = Atomic::new(&mut v as *mut _);
    let b = Atomic::new(Age(3));
    b.store(Age(7), Ordering::SeqCst);
    println!("{:?}", b.into_inner());

    let c: Atomic<bool> = Atomic::new(false);
    c.fetch_or(true, Ordering::SeqCst);
    println!("{:?}", c.into_inner());
}

#[derive(Debug)]
struct Age(u8);

impl Atom for Age {
    type Repr = u8;

    fn pack(self) -> Self::Repr {
        self.0
    }
    fn unpack(src: Self::Repr) -> Self {
        Self(src)
    }
}
