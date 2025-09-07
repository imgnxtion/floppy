    use clap::Parser;
    use std::fs;
    use std::fs::{File, OpenOptions};
    use std::io;
    use std::path::{Path, PathBuf};
    use std::process::{exit, Command};

    #[derive(Parser)]
    #[command(name = "meta")]
    #[command(about = "A lightweight tool to copy file contents (ASCII text only) or directory structure content (meta) to/from clipboard")]
    struct Cli {
        /// Paste from clipboard instead of copying
        #[arg(short)]
        paste: bool,
        /// Path to file or directory
        path: PathBuf,
    }

    fn main() {
        let cli = Cli::parse();

        if cli.paste {
            paste_to_path(&cli.path);
        } else {
            copy_from_path(&cli.path);
        }
    }

    fn copy_from_path(path: &Path) {
        let abs = match fs::canonicalize(path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error: could not resolve path '{}': {}", path.display(), e);
                exit(1);
            }
        };

        if abs.is_file() {
            // Check if file is text or binary
            if is_text_file(&abs) {
                // Copy contents
                match fs::read_to_string(&abs) {
                    Ok(contents) => {
                        copy_text_to_clipboard(&contents);
                        println!("✅ Copied contents of {} to clipboard", abs.display());
                    }
                    Err(e) => {
                        eprintln!("Error reading file '{}': {}", abs.display(), e);
                        exit(1);
                    }
                }
            } else {
                // Copy file reference
                copy_file_to_clipboard(&abs);
                println!("✅ Copied file {} to clipboard", abs.display());
            }
        } else if abs.is_dir() {
            // Copy file list
            match get_file_list(&abs) {
                Ok(list) => {
                    copy_text_to_clipboard(&list);
                    println!("✅ Copied directory structure of {} to clipboard", abs.display());
                }
                Err(e) => {
                    eprintln!("Error getting file list for '{}': {}", abs.display(), e);
                    exit(1);
                }
            }
        } else {
            eprintln!("Error: '{}' is neither a file nor a directory", abs.display());
            exit(1);
        }
    }

    fn paste_to_path(path: &Path) {
        let clipboard_contents = match get_clipboard_contents() {
            Ok(contents) => contents,
            Err(e) => {
                eprintln!("Error getting clipboard contents: {}", e);
                exit(1);
            }
        };

        // Try to parse as tree (for backward compatibility)
        let lines: Vec<String> = clipboard_contents.lines().map(|s| s.to_string()).collect();
        let entries = collect_entries(&lines);

        if !entries.is_empty() && entries.iter().any(|e| e.depth > 0 || e.is_dir) {
            // Looks like tree, create structure
            let opts = MaketreeOptions {
                dry_run: false,
                force: false,
                verbose: 0,
            };
            let base_path = PathBuf::from(path);
            if !base_path.exists() {
                if let Err(e) = fs::create_dir_all(&base_path) {
                    eprintln!("Error creating directory '{}': {}", base_path.display(), e);
                    exit(1);
                }
            }
            // Change to the directory
            if let Err(e) = std::env::set_current_dir(&base_path) {
                eprintln!("Error changing to directory '{}': {}", base_path.display(), e);
                exit(1);
            }
            if let Err(e) = run_maketree_with_entries(entries, opts) {
                eprintln!("Error creating structure: {}", e);
                exit(1);
            }
            println!("✅ Created directory structure at {}", path.display());
        } else {
            // Treat as list of paths
            for line in lines {
                if line.trim().is_empty() {
                    continue;
                }
                let is_dir = line.ends_with('/');
                let rel_path_str = if is_dir { &line[..line.len() - 1] } else { &line };
                let full_path = path.join(rel_path_str);
                if is_dir {
                    if let Err(e) = fs::create_dir_all(&full_path) {
                        eprintln!("Error creating dir '{}': {}", full_path.display(), e);
                        exit(1);
                    }
                } else {
                    if let Some(parent) = full_path.parent() {
                        if let Err(e) = fs::create_dir_all(parent) {
                            eprintln!("Error creating parent dirs for '{}': {}", full_path.display(), e);
                            exit(1);
                        }
                    }
                    if let Err(e) = fs::write(&full_path, "") {
                        eprintln!("Error creating file '{}': {}", full_path.display(), e);
                        exit(1);
                    }
                }
            }
            println!("✅ Created structure at {}", path.display());
        }
    }

    fn is_text_file(path: &Path) -> bool {
        // Simple check: try to read as UTF-8
        if let Ok(contents) = fs::read(path) {
            std::str::from_utf8(&contents).is_ok()
        } else {
            false
        }
    }

    fn copy_text_to_clipboard(text: &str) {
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to run pbcopy");

        use std::io::Write;
        child.stdin.as_mut().unwrap().write_all(text.as_bytes()).unwrap();
        child.wait().unwrap();
    }

    fn copy_file_to_clipboard(path: &Path) {
        let script = format!(
            r#"
    tell application "System Events"
        set the clipboard to (POSIX file "{}")
    end tell
    "#,
            path.display()
        );

        let status = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .status()
            .expect("Failed to run osascript");

        if !status.success() {
            eprintln!("Failed to copy file to clipboard");
            exit(1);
        }
    }

    fn get_file_list(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        // Get dirs
        let dir_output = Command::new("find")
            .arg(path)
            .arg("-type")
            .arg("d")
            .output()?;
        let file_output = Command::new("find")
            .arg(path)
            .arg("-type")
            .arg("f")
            .output()?;

        if !dir_output.status.success() || !file_output.status.success() {
            return Err("find command failed".into());
        }

        let mut paths = Vec::new();
        let prefix = path.to_string_lossy().to_string() + "/";

        for line in String::from_utf8(dir_output.stdout)?.lines() {
            if let Some(rel) = line.strip_prefix(&prefix) {
                paths.push(rel.to_string() + "/");
            } else if line == prefix.trim_end_matches('/') {
                paths.push(".".to_string() + "/");
            }
        }

        for line in String::from_utf8(file_output.stdout)?.lines() {
            if let Some(rel) = line.strip_prefix(&prefix) {
                paths.push(rel.to_string());
            }
        }

        paths.sort();
        Ok(paths.join("\n"))
    }

    fn get_clipboard_contents() -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("pbpaste").output()?;
        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Err("pbpaste failed".into())
        }
    }

    // ... existing code for maketree functions ...

    fn run_maketree_with_entries(entries: Vec<Entry>, opts: MaketreeOptions) -> Result<(), Box<dyn std::error::Error>> {
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

    #[derive(Debug)]
    struct MaketreeOptions {
        dry_run: bool,
        force: bool,
        verbose: u8,
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
                entries.push(Entry {
                    depth,
                    name,
                    is_dir,
                });
            } else {
                let (depth, name) = parse_indent_list_depth(raw);
                if name.is_empty() {
                    continue;
                }
                let is_dir = name.ends_with('/');
                entries.push(Entry {
                    depth,
                    name,
                    is_dir,
                });
            }
        }

        for i in 0..entries.len() {
            if entries[i].is_dir {
                continue;
            }
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
                    eprintln_v(
                        opts.verbose,
                        1,
                        format!("[INFO] Removing file to create dir: {}", path.display()),
                    );
                    if opts.dry_run {
                        println!("Would remove file: {}", path.display());
                    } else {
                        fs::remove_file(path)?;
                        fs::create_dir_all(path)?;
                    }
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        format!(
                            "File exists where directory expected: {} (use --force)",
                            path.display()
                        ),
                    ));
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
                    eprintln_v(
                        opts.verbose,
                        1,
                        format!("[INFO] Removing dir to create file: {}", path.display()),
                    );
                    if opts.dry_run {
                        println!("Would remove dir: {}", path.display());
                    } else {
                        fs::remove_dir_all(path)?;
                    }
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        format!(
                            "Directory exists where file expected: {} (use --force)",
                            path.display()
                        ),
                    ));
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
