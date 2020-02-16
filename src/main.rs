// Based on: https://gist.github.com/zroug/28605f45a662b483fb4a7c3545627f66

mod format;

use format::{Format, Formatter, Hunk, Hunks};
use git2::Delta;
use git2::DiffOptions;
use git2::Repository;
use std::collections::HashMap;
use std::env::set_current_dir;

struct FmtHunks<'a> {
    fmt: &'a Formatter,
    hunks: Hunks,
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

fn main() {
    let fmts: [Formatter; 1] = [Formatter::RustFmt(format::CallRustFmt {})];
    let mut ext_hunks = HashMap::with_capacity(fmts.len());

    for fmt in fmts.iter() {
        for ext in fmt.extensions() {
            let old = ext_hunks.insert(
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

    let repo = Repository::open_from_env().unwrap();
    set_current_dir(repo.workdir().unwrap()).unwrap();

    let head = repo.head().unwrap().peel_to_tree().unwrap();

    let mut diff_options = DiffOptions::new();
    diff_options.include_untracked(true);
    diff_options.recurse_untracked_dirs(true);
    diff_options.show_untracked_content(true);
    diff_options.context_lines(2);
    let diff = repo
        .diff_tree_to_workdir(Some(&head), Some(&mut diff_options))
        .unwrap();

    diff.foreach(
        &mut |_, _| true,
        Some(&mut |_, _| true),
        Some(&mut |delta, hunk| {
            let hunks = if let Some(ext) = delta.new_file().path().unwrap().extension() {
                if let Some(hunks) = ext_hunks.get_mut(ext.to_str().unwrap()) {
                    hunks
                } else {
                    return true;
                }
            } else {
                return true;
            };

            match delta.status() {
                Delta::Added
                | Delta::Modified
                | Delta::Renamed
                | Delta::Copied
                | Delta::Untracked
                | Delta::Ignored => {
                    let h = Hunk {
                        start: hunk.new_start(),
                        lines: hunk.new_lines(),
                    };
                    let path = delta.new_file().path().unwrap().to_path_buf();
                    if let Some(existing) = hunks.hunks.get_mut(&path) {
                        existing.push(h);
                    } else {
                        hunks.hunks.insert(path, vec![h]);
                    }
                    true
                }
                _ => true,
            }
        }),
        None,
    )
    .unwrap();

    let mut fmt_hunks: Vec<FmtHunks> = Vec::new();
    for hunk in ext_hunks {
        if let Some(hunks) = fmt_hunks.iter_mut().find(|n| n.fmt == hunk.1.fmt) {
            hunks.merge(hunk.1);
        } else {
            fmt_hunks.push(hunk.1);
        }
    }

    for fmt_hunk in fmt_hunks.into_iter().filter(|x| !x.hunks.is_empty()) {
        fmt_hunk.format();
    }
}
