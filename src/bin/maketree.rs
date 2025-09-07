use std::env;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Default, Debug)]
struct Options {
    dry_run: bool,
    force: bool,
    verbose: u8,
    input_file: Option<PathBuf>,
}

fn eprintln_v(v: u8, level: u8, msg: impl AsRef<str>) {
    if v >= level {
        eprintln!("{}", msg.as_ref());
    }
}

fn parse_args() -> Result<Options, String> {
    let mut opts = Options::default();
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-d" | "--dry-run" => opts.dry_run = true,
            "-f" | "--force" => opts.force = true,
            "-v" => opts.verbose = opts.verbose.saturating_add(1),
            "-vv" => opts.verbose = opts.verbose.saturating_add(2),
            "-vvv" => opts.verbose = 3,
            "-i" | "--file" => {
                let p = args
                    .next()
                    .ok_or_else(|| "--file requires a path".to_string())?;
                opts.input_file = Some(PathBuf::from(p));
            }
            _ => return Err(format!("Unknown option: {}", arg)),
        }
    }

    Ok(opts)
}

fn print_help() {
    println!("Usage: maketree [OPTIONS]\n\nRead a tree-like structure and create directories and files from it.\n\nInput can be: \n- output from `tree` (with or without `-F`)\n- a simple indented list (2 spaces per level) where directory names end with `/`\n\nIf neither `--file` nor stdin is provided, the program waits for stdin.\n\nOptions:\n  -h, --help        Show this help message\n  -d, --dry-run     Print actions without making changes\n  -f, --force       Replace conflicting files/dirs if needed\n  -i, --file FILE   Read input from file\n  -v, -vv, -vvv     Increase verbosity (1..3)\n\nExamples:\n  tree -F myproj | maketree\n  maketree --file structure.tree\n  cat <<'EOF' | maketree\n  app/\n    src/\n      main.rs\n    Cargo.toml\n  EOF");
}

#[derive(Debug, Clone)]
struct Entry {
    depth: usize,
    name: String,
    is_dir: bool, // may be refined by lookahead for tree-without-slash
}

fn is_stats_line(s: &str) -> bool {
    // Matches lines like: "3 directories, 2 files" or "1 directory, 0 files"
    let s = s.trim();
    let mut digits_seen = false;
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            digits_seen = true;
            break;
        }
    }
    digits_seen && s.contains("director") && s.contains("file")
}

fn parse_tree_style_depth(line: &str) -> Option<(usize, String)> {
    // For classic `tree` output, before the connector (├── / └──) there are groups of 4 chars: "│   " or "    ".
    if let Some(conn_pos) = line.find("── ") {
        let prefix = &line[..conn_pos];
        let mut i = 0usize;
        let mut depth = 0usize;
        let bytes = prefix.as_bytes();
        while i + 4 <= bytes.len() {
            let chunk = &prefix[i..i + 4];
            if chunk == "│   " || chunk == "    " {
                depth += 1;
                i += 4;
            } else {
                break;
            }
        }
        let name = line[conn_pos + 3..].trim().to_string();
        return Some((depth, name));
    }
    None
}

fn parse_indent_list_depth(line: &str) -> (usize, String) {
    // 2 spaces per indent level; directory lines end with '/'
    let leading = line.chars().take_while(|c| c.is_whitespace()).count();
    let name = line[leading..].trim().to_string();
    let depth = leading / 2; // heuristic, matches our simple list format
    (depth, name)
}

fn collect_entries(lines: &[String]) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();

    // First pass: parse depth and raw name, mark dir if name endswith '/'
    for raw in lines {
        if raw.trim().is_empty() || raw.trim() == "." || is_stats_line(raw) {
            continue;
        }

        if let Some((depth, name)) = parse_tree_style_depth(raw) {
            let is_dir = name.ends_with('/');
            entries.push(Entry { depth, name, is_dir });
        } else {
            let (depth, name) = parse_indent_list_depth(raw);
            if name.is_empty() { continue; }
            let is_dir = name.ends_with('/');
            entries.push(Entry { depth, name, is_dir });
        }
    }

    // Second pass: for `tree` outputs without `-F`, directories won't end with '/'.
    // Infer directories when the next line is deeper.
    for i in 0..entries.len() {
        if entries[i].is_dir { continue; }
        if i + 1 < entries.len() {
            if entries[i + 1].depth > entries[i].depth {
                entries[i].is_dir = true;
            }
        }
    }

    entries
}

fn ensure_dir(path: &Path, opts: &Options) -> io::Result<()> {
    if path.exists() {
        if path.is_file() {
            if opts.force {
                eprintln_v(opts.verbose, 1, format!("[INFO] Removing file to create dir: {}", path.display()));
                if opts.dry_run {
                    println!("Would remove file: {}", path.display());
                } else {
                    fs::remove_file(path)?;
                    fs::create_dir_all(path)?;
                }
            } else {
                return Err(io::Error::new(io::ErrorKind::AlreadyExists, format!(
                    "File exists where directory expected: {} (use --force)", path.display()
                )));
            }
        }
        // already a dir: ok
        Ok(())
    } else {
        if opts.dry_run {
            println!("Would mkdir -p {}", path.display());
            Ok(())
        } else {
            fs::create_dir_all(path)
        }
    }
}

fn touch_file(path: &Path, opts: &Options) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent, opts)?;
    }

    if path.exists() {
        if path.is_dir() {
            if opts.force {
                eprintln_v(opts.verbose, 1, format!("[INFO] Removing dir to create file: {}", path.display()));
                if opts.dry_run {
                    println!("Would remove dir: {}", path.display());
                } else {
                    fs::remove_dir_all(path)?;
                }
            } else {
                return Err(io::Error::new(io::ErrorKind::AlreadyExists, format!(
                    "Directory exists where file expected: {} (use --force)", path.display()
                )));
            }
        } else {
            // file exists, nothing to do
            return Ok(());
        }
    }

    if opts.dry_run {
        println!("Would touch {}", path.display());
        Ok(())
    } else {
        let _f: File = OpenOptions::new().create(true).write(true).open(path)?;
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = parse_args().map_err(|e| {
        eprintln!("{}", e);
        print_help();
        e
    })?;

    // Read input
    let mut input = String::new();
    if let Some(file) = opts.input_file.as_ref() {
        OpenOptions::new().read(true).open(file)?.read_to_string(&mut input)?;
    } else {
        // Read from stdin; if no data, block waiting for it
        io::stdin().read_to_string(&mut input)?;
    }

    let lines: Vec<String> = input.lines().map(|s| s.to_string()).collect();
    let entries = collect_entries(&lines);

    // Build using a stack of path components per depth
    let mut stack: Vec<String> = Vec::new();
    for (idx, ent) in entries.iter().enumerate() {
        // Maintain stack to current depth
        if ent.depth >= stack.len() {
            stack.resize(ent.depth, String::new());
        } else {
            stack.truncate(ent.depth);
        }

        let name = ent.name.trim_end_matches('/');
        let mut path = PathBuf::from(".");
        for comp in &stack {
            if !comp.is_empty() {
                path.push(comp);
            }
        }
        path.push(name);

        if ent.is_dir {
            eprintln_v(opts.verbose, 2, format!("[DEBUG] mkdir: {}", path.display()));
            ensure_dir(&path, &opts)?;
            // push this directory onto stack for children
            if ent.depth == stack.len() {
                stack.push(name.to_string());
            } else if ent.depth < stack.len() {
                if ent.depth == 0 {
                    if stack.is_empty() {
                        stack.push(name.to_string());
                    } else {
                        stack[0] = name.to_string();
                    }
                } else {
                    stack[ent.depth] = name.to_string();
                }
            }
        } else {
            eprintln_v(opts.verbose, 2, format!("[DEBUG] touch: {}", path.display()));
            touch_file(&path, &opts)?;
        }

        // Optional: if next entry is shallower or same depth, nothing to do; depth is managed above
        let _next = entries.get(idx + 1);
    }

    Ok(())
}

