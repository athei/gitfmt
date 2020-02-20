// Based on: https://gist.github.com/zroug/28605f45a662b483fb4a7c3545627f66

mod format;

use format::{Formatters, Hunk};
use git2::Delta;
use git2::DiffOptions;
use git2::Repository;
use std::env::set_current_dir;

fn main() {
    let formatters = format::construct_repo();
    let mut formatters = Formatters::new(&formatters);

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
                if let Some(hunks) = formatters.fmts.get_mut(ext.to_str().unwrap()) {
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

    formatters.format();
}
