//! This simple example shows how to use `Atomic` with primitive types (`u32`
//! and `bool` in this case).

use std::{
    sync::Arc,
    thread,
    time::Duration,
};
use atomig::{Atomic, Ordering};


fn main() {
    // `Atomic<u32>`
    let a = Atomic::new(3u32);
    a.store(27, Ordering::SeqCst);
    println!("{:?}", a);


    // `Atomic<bool>`
    let b = Arc::new(Atomic::new(false));
    {
        let b = b.clone();
        thread::spawn(move || {
            while b.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst).is_err() {}
            println!("Reset it to false!");
        });
    }
    thread::sleep(Duration::from_millis(10));
    b.fetch_or(true, Ordering::SeqCst);

    thread::sleep(Duration::from_millis(2));
    println!("{}", b.load(Ordering::SeqCst));
}
