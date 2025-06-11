
use cli::parse;

use patch::Patch;

mod cli;

fn main() {
    let (_, patches, args) = match parse() {
        Ok(result) => result,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };

    let mut filtered_patches = vec![];

    for patch in patches {
        let mut filtered_hunks: Vec<patch::Hunk> = vec![];

        for hunk in patch.hunks {
            if lines_pass(&hunk.lines, &args) {
                filtered_hunks.push(hunk);
            }
        }

        if !filtered_hunks.is_empty() {
            filtered_patches.push(Patch {
                hunks: filtered_hunks,
                old: patch.old,
                new: patch.new,
                end_newline: patch.end_newline,
            });
        }
    }

    if args.files_with_matches {
        for patch in filtered_patches {
            let path = patch.new.path;
            println!("{}", path.chars().skip(2).collect::<String>());
        }

        return;
    }

    for patch in filtered_patches {
        println!("{}", patch);
    }
}

fn lines_pass(lines: &Vec<patch::Line>, args: &cli::Params) -> bool {
    let mut contains_grep = false;

    for line in lines {
        let line_matches = match line {
            patch::Line::Add(ref content) | patch::Line::Remove(ref content) => {
                if args.ignore_case {
                    content.to_lowercase().contains(&args.grep.to_lowercase())
                } else {
                    content.contains(&args.grep)
                }
            }
            _ => false,
        };

        if line_matches {
            contains_grep = true;
        }
    }

    if args.invert_match {
        !contains_grep
    } else {
        contains_grep
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_params(grep: &str, invert: bool, ignore: bool) -> cli::Params {
        cli::Params {
            grep: grep.to_string(),
            invert_match: invert,
            ignore_case: ignore,
            files_with_matches: false,
        }
    }

    #[test]
    fn match_found() {
        let args = build_params("foo", false, false);
        let lines = vec![
            patch::Line::Add("foo bar"),
            patch::Line::Remove("baz"),
        ];
        assert!(lines_pass(&lines, &args));
    }

    #[test]
    fn no_match() {
        let args = build_params("foo", false, false);
        let lines = vec![patch::Line::Add("bar")];
        assert!(!lines_pass(&lines, &args));
    }

    #[test]
    fn ignore_case() {
        let args = build_params("FOO", false, true);
        let lines = vec![patch::Line::Add("foo")];
        assert!(lines_pass(&lines, &args));
    }

    #[test]
    fn invert_match() {
        let args = build_params("foo", true, false);
        let lines = vec![patch::Line::Add("bar")];
        assert!(lines_pass(&lines, &args));
    }
}
