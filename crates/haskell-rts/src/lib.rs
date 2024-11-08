use std::env;
use std::ffi::{c_char, CString};

#[allow(warnings)]
pub mod raw;

pub struct Haskell {
    inner: HaskellInternal,
}

impl Haskell {
    pub fn init(args: env::Args) -> Self {
        Self {
            inner: HaskellInternal::init(args),
        }
    }
}

struct HaskellInternal;

impl HaskellInternal {
    pub fn init(args: env::Args) -> Self {
        let mut args = args
            .map(|arg| CString::new(arg).unwrap())
            .map(|arg| arg.into_raw())
            .collect::<Vec<*mut c_char>>();
        unsafe { raw::hs_init(args.len() as *mut i32, &mut args.as_mut_ptr()) }
        Self
    }
}

impl Drop for HaskellInternal {
    fn drop(&mut self) {
        unsafe {
            raw::hs_exit();
        }
    }
}
