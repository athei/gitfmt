// Based on: https://gist.github.com/zroug/28605f45a662b483fb4a7c3545627f66

use git2::Delta;
use git2::DiffOptions;
use git2::Repository;
use std::collections::HashSet;
use std::env::set_current_dir;
use std::process::Command;

fn main() {
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

    let mut changed_lines = Vec::new();
    diff.foreach(
        &mut |_, _| true,
        Some(&mut |_, _| true),
        Some(&mut |delta, hunk| {
            if delta.new_file().path().unwrap().extension() != Some("rs".as_ref()) {
                return true;
            }
            match delta.status() {
                Delta::Unmodified
                | Delta::Deleted
                | Delta::Typechange
                | Delta::Unreadable
                | Delta::Conflicted => true,
                Delta::Added
                | Delta::Modified
                | Delta::Renamed
                | Delta::Copied
                | Delta::Untracked
                | Delta::Ignored => {
                    changed_lines.push((
                        delta.new_file().path().unwrap().to_path_buf(),
                        hunk.new_start(),
                        hunk.new_start() + hunk.new_lines() - 1,
                    ));
                    true
                }
            }
        }),
        None,
    )
    .unwrap();

    if changed_lines.is_empty() {
        return;
    }

    let mut changes_json = String::new();
    let mut changed_paths = HashSet::new();
    changes_json.push('[');
    for (path, start, end) in changed_lines {
        changes_json.push_str(&format!(
            "{{\"file\":\"{}\",\"range\":[{},{}]}},",
            path.to_str().unwrap(),
            start,
            end
        ));
        changed_paths.insert(path);
    }
    changes_json.pop();
    changes_json.push(']');

    Command::new("rustfmt")
        .args(&[
            "+nightly",
            "--unstable-features",
            "--file-lines",
            &changes_json,
            "--skip-children",
        ])
        .args(changed_paths)
        .status()
        .unwrap();
}
