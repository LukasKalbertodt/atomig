//! This example shows how you can use `Atomic<T>` with your own types by
//! deriving `Atom`.

use std::{
    sync::Arc,
    thread,
    time::Duration,
};
use atomig::{Atom, Atomic, AtomLogic, Ordering};


#[derive(Debug, PartialEq, Atom)]
#[repr(u8)]
enum Animal {
    Cat,
    Dog,
    Fox,
}

#[derive(Debug, PartialEq, Atom, AtomLogic)]
struct BitSet(u16);


fn main() {
    // Example with `Animal`
    let animal = Arc::new(Atomic::new(Animal::Cat));
    {
        let animal = animal.clone();
        thread::spawn(move || {
            while animal
                .compare_exchange(Animal::Dog, Animal::Fox, Ordering::SeqCst, Ordering::SeqCst)
                .is_err()
            {
                thread::sleep(Duration::from_millis(2));
            }
            println!("Changed dog to fox!");
        });
    }
    thread::sleep(Duration::from_millis(10));
    animal.store(Animal::Dog, Ordering::SeqCst);
    println!("Changed to dog!");

    thread::sleep(Duration::from_millis(10));
    println!("Final animal: {:?}", animal);


    // Example with `BitSet`
    let integer_set = Arc::new(Atomic::new(BitSet(0b11001100_11110000)));
    {
        let integer_set = integer_set.clone();
        thread::spawn(move || {
            let mut current = 1;
            while integer_set.load(Ordering::SeqCst) != BitSet(0xFFFF) {
                integer_set.fetch_or(BitSet(current), Ordering::SeqCst);
                current <<= 1;
            }
            println!("Set all bits!");
        });
    }
    thread::sleep(Duration::from_millis(10));
    println!("{:b}", integer_set.load(Ordering::SeqCst).0);
}
