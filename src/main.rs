
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

    fn make_params(grep: &str, invert: bool, icase: bool) -> cli::Params {
        cli::Params {
            grep: grep.to_string(),
            invert_match: invert,
            ignore_case: icase,
            files_with_matches: false,
        }
    }

    #[test]
    fn match_simple() {
        let lines = vec![patch::Line::Add("hello world")];
        let params = make_params("hello", false, false);
        assert!(lines_pass(&lines, &params));
    }

    #[test]
    fn match_ignore_case() {
        let lines = vec![patch::Line::Add("Hello")];
        let params = make_params("hello", false, true);
        assert!(lines_pass(&lines, &params));
    }

    #[test]
    fn invert_when_no_match() {
        let lines = vec![patch::Line::Add("foo")];
        let params = make_params("bar", true, false);
        assert!(lines_pass(&lines, &params));
    }

    #[test]
    fn invert_with_match() {
        let lines = vec![patch::Line::Add("foo")];
        let params = make_params("foo", true, false);
        assert!(!lines_pass(&lines, &params));
    }

    #[test]
    fn no_match_no_invert() {
        let lines = vec![patch::Line::Add("foo")];
        let params = make_params("bar", false, false);
        assert!(!lines_pass(&lines, &params));
    }
}
