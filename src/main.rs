use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, exit};

fn main() {
    let path = match env::args().nth(1) {
        Some(p) => PathBuf::from(p),
        None => {
            eprintln!("Usage: copyfile-link path/to/file.md");
            exit(1);
        }
    };

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

