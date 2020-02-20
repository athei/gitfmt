// Based on: https://gist.github.com/zroug/28605f45a662b483fb4a7c3545627f66

mod format;
mod git;

fn main() {
    let formatters = format::construct_repo();
    let mut formatters = format::Formatters::new(&formatters);

    git::collect_hunks(&mut formatters);
    formatters.format();
}
