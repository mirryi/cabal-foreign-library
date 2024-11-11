use std::io;

/// Type for errors that may occur.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cabal invocation error")]
    CabalError(#[source] InvocationError),
    #[error("ghc-pkg invocation error")]
    GHCPkgError(#[source] InvocationError),

    #[error("build error")]
    BuildError(#[source] Option<io::Error>),
}

/// An error that occurs when invoking `cabal` or `ghc-pkg`.
#[derive(Debug, thiserror::Error)]
pub enum InvocationError {
    #[error("resolution error")]
    ResolutionError(#[from] which::Error),
    #[error("i/o error")]
    IoError(#[from] io::Error),
}
