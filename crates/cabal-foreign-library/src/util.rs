use std::ffi::OsStr;
use std::process::Command;
use std::{env, io};

use camino::Utf8PathBuf;

use crate::InvocationError;

pub fn package() -> String {
    env::var("CARGO_PKG_NAME").unwrap()
}

pub fn out_dir() -> String {
    env::var("OUT_DIR").unwrap()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub const DYLIB_EXT: &str = "so";
#[cfg(target_os = "macos")]
pub const DYLIB_EXT: &str = "dylib";
#[cfg(target_os = "windows")]
pub const DYLIB_EXT: &str = "dll";

pub trait CommandStdoutExt {
    fn stdout(&mut self) -> Result<String, io::Error>;

    fn stdout_trim(&mut self) -> Result<String, io::Error> {
        self.stdout().map(|stdout| stdout.trim_end().to_string())
    }
}

impl CommandStdoutExt for Command {
    fn stdout(&mut self) -> Result<String, io::Error> {
        let output = self.output()?;
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        Ok(String::from(stdout))
    }
}

pub fn which(name: impl AsRef<OsStr>) -> Result<Utf8PathBuf, InvocationError> {
    let path = which::which(name)?;
    Ok(Utf8PathBuf::from_path_buf(path).unwrap())
}
