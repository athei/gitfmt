use std::collections::HashMap;
use std::fmt::{self, Display};
use std::path::PathBuf;

mod callrustfmt;

pub use callrustfmt::CallRustFmt;

pub type Hunks = HashMap<PathBuf, Vec<Hunk>>;

pub struct Hunk {
    pub start: u32,
    pub lines: u32,
}

pub trait Format: fmt::Display + std::cmp::PartialEq {
    fn extensions(&self) -> &'static [&'static str];
    fn format(&self, p: Hunks) -> Result<(), String>;
}

#[derive(PartialEq)]
pub enum Formatter {
    RustFmt(CallRustFmt),
}

impl Display for Formatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RustFmt(x) => x.fmt(f),
        }
    }
}

impl Format for Formatter {
    fn extensions(&self) -> &'static [&'static str] {
        match self {
            Self::RustFmt(x) => x.extensions(),
        }
    }

    fn format(&self, p: Hunks) -> Result<(), String> {
        match self {
            Self::RustFmt(x) => x.format(p),
        }
    }
}
