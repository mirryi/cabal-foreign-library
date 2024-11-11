# example-basic

This is a basic of example of how to use `cabal-foreign-library`.

In this example the directories of the required shared dynamic libraries (`example-basic` itself and
Haskell runtime libraries) are added to resulting executables' runpaths (see calls to `link` and
`link_system` in [`build.rs`](./build.rs)).
This tool does not handle installation of the shared libraries into locations in which they may be
found by the Rust executables.
