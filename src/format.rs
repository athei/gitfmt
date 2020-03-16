use std::collections::HashMap;
use std::fmt::{self, Display};
use std::path::PathBuf;

mod callrustfmt;

pub struct Hunk {
    pub start: u32,
    pub lines: u32,
}

pub struct Formatters<'a> {
    repo: &'a [Formatter],
    fmts: HashMap<&'static str, FmtHunks<'a>>,
}

type Hunks = HashMap<PathBuf, Vec<Hunk>>;

struct FmtHunks<'a> {
    fmt: &'a Formatter,
    hunks: Hunks,
}

impl<'a> Formatters<'a> {
    pub fn ext_supported(&self, ext: &str) -> bool {
        self.fmts.contains_key(ext)
    }

    pub fn add_hunk(&mut self, ext: &str, path: PathBuf, hunk: Hunk) {
        if let Some(hunks) = self.fmts.get_mut(ext) {
            hunks.hunks.entry(path).or_insert(Vec::new()).push(hunk);
        }
    }
}

impl<'a> Formatters<'a> {
    pub fn new(repo: &'a [Formatter]) -> Self {
        let mut fmts = Formatters {
            repo,
            fmts: HashMap::new(),
        };

        for fmt in fmts.repo.iter() {
            for ext in fmt.extensions() {
                let old = fmts.fmts.insert(
                    *ext,
                    FmtHunks {
                        fmt,
                        hunks: Hunks::new(),
                    },
                );
                if let Some(old) = old {
                    panic!(
                        "Formatter {} tried to add extension already added by {}",
                        *ext, old.fmt
                    );
                }
            }
        }
        fmts
    }

    pub fn format(self) {
        let mut merged: Vec<FmtHunks> = Vec::new();
        for hunk in self.fmts {
            if let Some(hunks) = merged.iter_mut().find(|n| n.fmt == hunk.1.fmt) {
                hunks.merge(hunk.1);
            } else {
                merged.push(hunk.1);
            }
        }

        for fmt_hunk in merged.into_iter().filter(|x| !x.hunks.is_empty()) {
            fmt_hunk.format();
        }
    }
}

impl<'a> FmtHunks<'a> {
    fn merge(&mut self, o: Self) {
        assert!(self.fmt == o.fmt);
        for mut hunks in o.hunks {
            if let Some(p) = self.hunks.get_mut(&hunks.0) {
                p.append(&mut hunks.1)
            } else {
                self.hunks.insert(hunks.0, hunks.1);
            }
        }
    }

    fn format(self) {
        self.fmt.format(self.hunks).unwrap();
    }
}

pub trait Format: fmt::Display + std::cmp::PartialEq {
    fn extensions(&self) -> &'static [&'static str];
    fn format(&self, p: Hunks) -> Result<(), String>;
}

#[derive(PartialEq)]
pub enum Formatter {
    RustFmt(callrustfmt::CallRustFmt),
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

pub fn construct_repo() -> Vec<Formatter> {
    vec![Formatter::RustFmt(callrustfmt::CallRustFmt {})]
}
