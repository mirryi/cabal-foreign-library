use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cabal invocation error")]
    CabalError(#[source] InvocationError),
    #[error("ghc-pkg invocation error")]
    GHCPkgError(#[source] InvocationError),

    #[error("build error")]
    BuildError(#[source] Option<io::Error>),
    #[error("bindings error")]
    BindingsError(#[from] BindingsError),
}

#[derive(Debug, thiserror::Error)]
pub enum InvocationError {
    #[error("resolution error")]
    ResolutionError(#[from] which::Error),
    #[error("i/o error")]
    IoError(#[from] io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum BindingsError {
    #[error("i/o error")]
    IoError(#[from] io::Error),

    #[error("bindgen error")]
    BindgenError(#[from] bindgen::BindgenError),
}
