use super::{Format, Hunks};
use std::fmt;
use std::process::Command;

const EXTENSIONS: [&str; 1] = ["rs"];
const NAME: &str = "CallRustFmt";

pub struct CallRustFmt;

impl Format for CallRustFmt {
    fn extensions(&self) -> &'static [&'static str] {
        &EXTENSIONS
    }

    fn format(&self, hunks: Hunks) -> Result<(), String> {
        let mut changes_json = String::new();
        changes_json.push('[');
        for (path, hunks) in hunks.iter() {
            for hunk in hunks {
                changes_json.push_str(&format!(
                    "{{\"file\":\"{}\",\"range\":[{},{}]}},",
                    path.to_str().unwrap(),
                    hunk.start,
                    hunk.start + hunk.lines + 1,
                ));
            }
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
            .args(hunks.keys())
            .status()
            .unwrap();
        Ok(())
    }
}

impl fmt::Display for CallRustFmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", NAME)
    }
}

impl std::cmp::PartialEq for CallRustFmt {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}
