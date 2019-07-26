Atomig: generic and convenient `std` atomics
============================================

[![Build Status](https://img.shields.io/travis/LukasKalbertodt/atomig/master.svg)](https://travis-ci.org/LukasKalbertodt/atomig)
[![crates.io version](https://img.shields.io/crates/v/atomig.svg)](https://crates.io/crates/atomig)
[![docs](https://docs.rs/atomig/badge.svg)](https://docs.rs/atomig)

Offers `Atomic<T>` that can be used with primitive and custom types.
*However*, it only works with types that can actually use atomic operations: a lock-based fallback for other types is not used!
This crate is based on `std`'s atomics and therefore does not contain any `unsafe` code!
This crate also does not have any dependencies by default.
If you enable the `serde` feature, then this crate will depend on `serde` and `Serialize` / `Deserialize` will be
implemented for `Atomic<T>` when appropriate, using sequentially-consistent ordering.

Simple example with primitive types:

```rust
use atomig::{Atomic, Ordering};

let x = Atomic::new(27); // `Atomic<i32>`
x.store(39, Ordering::SeqCst);
```

This works with almost all primitive types, including `f32`, `f64` and `char`.

You can automatically derive `Atom` for your own enum or struct types to use them in `Atomic<T>`.
There are some limitations, however.

```rust
// Requires the 'derive' feature:
//     atomig = { version = "_", features = ["derive"] }
use atomig::{Atom, Atomic, Ordering};

#[derive(Atom)]
#[repr(u8)]
enum Animal { Dog, Cat, Fox }

let animal = Atomic::new(Animal::Cat);
animal.store(Animal::Fox, Ordering::SeqCst);

#[derive(Atom)]
struct Port(u16);

let port = Atomic::new(Port(80));
port.store(Port(8080), Ordering::SeqCst);
```

For more examples and information see **[the documentation](https://docs.rs/atomig)**.

---

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
