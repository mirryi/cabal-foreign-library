use std::ptr::null_mut;

use example_basic::{foo, hs_exit, hs_init};

fn main() {
    unsafe {
        hs_init(null_mut(), null_mut());
        println!("Haskell returns: {}", foo(42));
        hs_exit();
    }
}
