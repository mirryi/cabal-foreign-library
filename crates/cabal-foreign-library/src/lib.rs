mod error;
mod util;

use std::process::Command;
use std::{fs, str};

use camino::{Utf8Path, Utf8PathBuf};
use regex::Regex;
use util::{CommandStdoutExt, CARGO_PKG_NAME, DYLIB_EXT, OUT_DIR};

pub use error::*;

#[derive(Debug)]
pub struct Build {
    cabal: Utf8PathBuf,
    ghc_pkg: Utf8PathBuf,
    rts_version: RTSVersion,
}

#[derive(Debug)]
pub struct Lib<'b> {
    build: &'b Build,
    path: Utf8PathBuf,
    hs_deps: Vec<HSDep>,
}

#[derive(Debug, Clone, Copy)]
enum HSDep {
    Ghc,
    Base,
}

pub type Bindings = bindgen::Bindings;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy)]
pub enum RTSVersion {
    NonThreaded,
    NonThreadedL,
    NonThreadedDebug,
    Threaded,
    ThreadedL,
    ThreadedDebug,
}

impl RTSVersion {
    fn default() -> Self {
        Self::NonThreaded
    }
}

impl Build {
    /// Construct a new instance with a default configuration.
    pub fn new() -> Result<Self> {
        let cabal = util::which("cabal").map_err(Error::CabalError)?;
        let ghc_pkg = util::which("ghc-pkg").map_err(Error::GHCPkgError)?;
        Ok(Self {
            cabal,
            ghc_pkg,
            rts_version: RTSVersion::default(),
        })
    }

    /// Set the `cabal` binary.
    pub fn use_cabal(&mut self, path: impl AsRef<Utf8Path>) -> &mut Self {
        self.cabal = path.as_ref().to_path_buf();
        self
    }

    /// Set the `ghc-pkg` binary.
    pub fn use_ghc_pkg(&mut self, path: impl AsRef<Utf8Path>) -> &mut Self {
        self.ghc_pkg = path.as_ref().to_path_buf();
        self
    }

    /// Set the version of the GHC RTS.
    ///
    /// By default, [`RTSVersion::NonThreaded`] is used.
    pub fn use_rts(&mut self, rts_version: RTSVersion) -> &mut Self {
        self.rts_version = rts_version;
        self
    }

    /// Build the foreign library with cabal.
    pub fn build(&mut self) -> Result<Lib> {
        // build
        let status = self
            .cabal_cmd("build")
            .status()
            .map_err(|err| Error::BuildError(Some(err)))?;
        if !status.success() {
            return Err(Error::BuildError(None));
        }

        // find the dylib file
        let path = self
            .cabal_cmd("list-bin")
            .arg(&*CARGO_PKG_NAME)
            .stdout_trim()
            .map(Utf8PathBuf::from)
            .map_err(|err| Error::BuildError(Some(err)))?;

        Ok(Lib {
            build: self,
            path,
            // TODO somehow pull necessary dependencies from cabal? or just link all system
            // dependencies?
            hs_deps: vec![HSDep::Ghc, HSDep::Base],
        })
    }

    fn cabal_cmd(&self, cmd: &str) -> Command {
        let mut cabal = Command::new(&self.cabal);
        cabal.args([cmd, "--builddir", &OUT_DIR]);
        cabal
    }

    fn ghc_pkg_cmd(&self, cmd: &str) -> Command {
        let mut ghc_pkg = Command::new(&self.ghc_pkg);
        ghc_pkg.args([cmd]);
        ghc_pkg
    }
}

impl<'b> Lib<'b> {
    /// Link the crate to the dynamic library.
    pub fn link(&self) -> Result<()> {
        let dir = self.path.parent().unwrap();
        println!("cargo::rustc-link-search=native={}", dir);
        println!("cargo::rustc-link-lib=dylib={}", *CARGO_PKG_NAME);

        Ok(())
    }

    /// Generate Rust bindings for the dynamic library.
    pub fn bindings(&self) -> Result<Bindings> {
        // find GHC RTS headers to be included
        let rts_headers = self
            .build
            .ghc_pkg_cmd("field")
            .args(["rts", "include-dirs", "--simple-output"])
            .stdout_trim()
            .map(Utf8PathBuf::from)
            .map_err(BindingsError::IoError)?;

        // find the stub file
        let stub = self
            .path
            .parent()
            .unwrap()
            .join(format!("{}-tmp", *CARGO_PKG_NAME))
            .join("Lib_stub.h");

        // invoke bindgen
        let bindings = bindgen::Builder::default()
            .clang_args(["-isystem", rts_headers.as_str()])
            .header(stub)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .map_err(BindingsError::BindgenError)?;

        Ok(bindings)
    }

    /// Link the crate to the Haskell system libraries.
    pub fn link_system(&self) -> Result<()> {
        // retrieve dynamic libraries directory.
        let ghc_lib_dir = self
            .build
            .ghc_pkg_cmd("field")
            .args(["rts", "dynamic-library-dirs", "--simple-output"])
            .stdout_trim()
            .map_err(InvocationError::IoError)
            .map_err(Error::GHCPkgError)?;
        let ghc_lib_dir = fs::canonicalize(ghc_lib_dir).unwrap();

        // regexes to match the necessary system dependencies.
        let version_regex = Regex::new(r"((\d+)\.)+?(\d+)").unwrap();

        let non_rts_prefixes = self
            .hs_deps
            .iter()
            .map(HSDep::prefix)
            .collect::<Vec<_>>()
            .join("|");
        let non_rts_regex = Regex::new(&format!(
            r"^lib({prefix})-({version})-ghc({version})\.{ext}$",
            prefix = non_rts_prefixes,
            version = version_regex,
            ext = DYLIB_EXT
        ))
        .unwrap();

        let rts_suffix = self.build.rts_version.suffix();
        let rts_regex = Regex::new(&format!(
            r"^libHSrts-({version})({suffix})-ghc({version})\.{ext}$",
            version = version_regex,
            suffix = rts_suffix,
            ext = DYLIB_EXT
        ))
        .unwrap();

        println!("{}", non_rts_regex);
        println!("{}", rts_regex);

        // link matching library files
        println!("cargo:rustc-link-search=native={}", ghc_lib_dir.display());
        for entry in fs::read_dir(ghc_lib_dir).unwrap() {
            let entry = entry.unwrap();

            match entry.file_name().to_str() {
                Some(i) => {
                    if non_rts_regex.is_match(i) || rts_regex.is_match(i) {
                        // get rid of lib from the file name
                        let temp = i.split_at(3).1;
                        // get rid of the .so from the file name
                        let trimmed = temp.split_at(temp.len() - DYLIB_EXT.len() - 1).0;

                        println!("cargo:rustc-link-lib=dylib={}", trimmed);
                    }
                }
                _ => panic!("Unable to link ghc libs"),
            }
        }

        Ok(())
    }
}

impl RTSVersion {
    pub fn suffix(&self) -> &str {
        match self {
            RTSVersion::NonThreaded => "",
            RTSVersion::NonThreadedL => "_l",
            RTSVersion::NonThreadedDebug => "_debug",
            RTSVersion::Threaded => "_thr",
            RTSVersion::ThreadedL => "_thr_l",
            RTSVersion::ThreadedDebug => "_thr_debug",
        }
    }
}

impl HSDep {
    pub fn prefix(&self) -> &str {
        match self {
            HSDep::Ghc => "HSghc",
            HSDep::Base => "HSbase",
        }
    }
}
