use clap::{Parser, Subcommand, CommandFactory};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, exit};

#[derive(Parser)]
#[command(name = "floppy")]
#[command(about = "A tool to copy files/directories and create directory structures")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Copy a file path to clipboard
    Copyfile {
        /// Path to the file to copy
        path: PathBuf,
    },
    /// Create directory structure from tree output
    Maketree {
        /// Input file (optional, reads from stdin if not provided)
        #[arg(short, long)]
        file: Option<PathBuf>,
        /// Dry run: print actions without making changes
        #[arg(short = 'd', long)]
        dry_run: bool,
        /// Force: replace conflicting files/dirs
        #[arg(short, long)]
        force: bool,
        /// Verbosity level (0-3)
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Copyfile { path }) => {
            copyfile(path);
        }
        Some(Commands::Maketree { file, dry_run, force, verbose }) => {
            maketree(file, dry_run, force, verbose);
        }
        None => {
            // No subcommand provided, show help
            let _ = Cli::command().print_help();
        }
    }
}

fn copyfile(path: PathBuf) {
    let abs = match fs::canonicalize(&path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: could not resolve file path '{}': {}", path.display(), e);
            exit(1);
        }
    };

    if !abs.is_file() {
        eprintln!("Error: '{}' is not a file", abs.display());
        exit(1);
    }

    let script = format!(r#"
tell application "System Events"
    set the clipboard to (POSIX file "{}")
end tell
"#, abs.display());

    let status = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status()
        .expect("Failed to run osascript");

    if status.success() {
        println!("✅ File URL copied to clipboard: {}", abs.display());
    } else {
        eprintln!("❌ Failed to copy file URL to clipboard.");
        exit(1);
    }
}

fn maketree(input_file: Option<PathBuf>, dry_run: bool, force: bool, verbose: u8) {
    let opts = MaketreeOptions {
        dry_run,
        force,
        verbose,
        input_file,
    };

    if let Err(e) = run_maketree(opts) {
        eprintln!("{}", e);
        exit(1);
    }
}

#[derive(Debug)]
struct MaketreeOptions {
    dry_run: bool,
    force: bool,
    verbose: u8,
    input_file: Option<PathBuf>,
}

fn run_maketree(opts: MaketreeOptions) -> Result<(), Box<dyn std::error::Error>> {
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

fn eprintln_v(v: u8, level: u8, msg: impl AsRef<str>) {
    if v >= level {
        eprintln!("{}", msg.as_ref());
    }
}

#[derive(Debug, Clone)]
struct Entry {
    depth: usize,
    name: String,
    is_dir: bool,
}

fn is_stats_line(s: &str) -> bool {
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
    let leading = line.chars().take_while(|c| c.is_whitespace()).count();
    let name = line[leading..].trim().to_string();
    let depth = leading / 2;
    (depth, name)
}

fn collect_entries(lines: &[String]) -> Vec<Entry> {
    let mut entries: Vec<Entry> = Vec::new();

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

fn ensure_dir(path: &Path, opts: &MaketreeOptions) -> io::Result<()> {
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

fn touch_file(path: &Path, opts: &MaketreeOptions) -> io::Result<()> {
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
