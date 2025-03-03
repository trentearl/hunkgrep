
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
