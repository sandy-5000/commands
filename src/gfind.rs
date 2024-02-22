use std::cmp::max;
use std::env;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process;

fn pop_last_arg(args: &mut Vec<String>) -> String {
    match args.pop() {
        Some(arg) => arg,
        _ => "".to_string(),
    }
}

fn set_flags(args: &mut Vec<String>, flags: &mut [bool]) {
    for flag in args.into_iter() {
        let flag_bytes: Vec<_> = flag.bytes().collect();
        let size = flag_bytes.len();
        if size == 0 {
            continue;
        }
        if size == 1 || flag_bytes[0] != 45 {
            eprintln!("Error at {}", flag);
            process::exit(1);
        }
        for i in 1..size {
            flags[flag_bytes[i] as usize] = true;
        }
    }
}

fn search_file(
    file_path: &str,
    pattern: &str,
    flags: &[bool],
) -> io::Result<(usize, Vec<(usize, String)>)> {
    let file = fs::File::open(file_path)?;
    let reader = BufReader::new(file);
    let mut matched_lines = Vec::new();
    let mut max_number = 0usize;
    let case_sensitive = flags[67] || flags[99];
    let pattern = if case_sensitive {
        pattern.to_string()
    } else {
        pattern.to_lowercase()
    };

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        let line_to_match = if case_sensitive {
            line.clone()
        } else {
            line.to_lowercase()
        };
        if line_to_match.contains(&pattern) {
            matched_lines.push((index + 1, line));
            max_number = max(max_number, index + 1);
        }
    }

    let length = max_number.to_string().len();

    Ok((length, matched_lines))
}

fn search_directory(dir: &str, root_dir: &str, pattern: &str, flags: &[bool]) {
    let path = Path::new(dir);

    if path.is_file() {
        let file_path = match path.strip_prefix(root_dir) {
            Ok(relative_path) => relative_path.to_string_lossy().into_owned(),
            _ => path.to_string_lossy().into_owned(),
        };
        let absolute_file_path = path.to_string_lossy().into_owned();
        if let Ok((number_format_length, matched_lines)) =
            search_file(&absolute_file_path, pattern, flags)
        {
            if matched_lines.len() == 0 {
                return;
            }
            if file_path != "" {
                println!("------ File: {}", file_path);
            }
            for (line_no, line) in matched_lines {
                println!(
                    "{:>width$}) {}",
                    line_no,
                    line,
                    width = number_format_length
                );
            }
            println!();
        }
        return;
    } else if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                search_directory(&path.to_string_lossy(), root_dir, pattern, flags);
            }
        }
    } else {
        eprintln!("Failed to read directory '{}'", root_dir);
    }
}

fn main() {
    let mut flags: [bool; 256] = [false; 256];
    let mut args: Vec<String> = env::args().skip(1).collect();

    if args.len() == 0 {
        eprintln!("command to scan files");
        process::exit(0);
    }

    if args.len() < 2 {
        eprintln!("Should at least proved 'folder|file path' and 'pattern'");
        process::exit(1);
    }

    let directory = pop_last_arg(&mut args);
    let pattern = pop_last_arg(&mut args);

    set_flags(&mut args, &mut flags);

    search_directory(&directory, &directory, &pattern, &flags);
}
