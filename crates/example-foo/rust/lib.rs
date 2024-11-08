#![allow(warnings)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod test {
    use crate::{foo, hs_exit, hs_init};

    #[test]
    fn test() {
        unsafe {
            hs_init(std::ptr::null_mut(), std::ptr::null_mut());

            assert_eq!(foo(0), 0);
            assert_eq!(foo(1), 1);
            assert_eq!(foo(7), 7);

            hs_exit();
        }
    }
}
