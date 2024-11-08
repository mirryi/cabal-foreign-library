use std::env;
use std::path::PathBuf;

use cabal_foreign_library::Build;

fn main() {
    let mut cabal = Build::new().unwrap();

    // build cabal project
    let lib = cabal.build().expect("failed to build cabal project");

    // generate and write bindings
    let bindings_file = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    lib.bindings()
        .expect("failed to generate bindings")
        .write_to_file(bindings_file)
        .expect("failed to write bindings");

    // link the library
    lib.link().expect("failed to link cabal library");

    // link the system dependencies
    lib.link_system()
        .expect("failed to link Haskell system libraries");
}
