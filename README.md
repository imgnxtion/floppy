# copyfile

A command-line utility for macOS that copies a file reference to the clipboard. When pasted, applications will receive the actual file, not just the text path.

> **Note:** This tool is part of the modular `taku` ecosystem of CLI utilities. See the [Part of the `taku` Ecosystem](#part-of-the-taku-ecosystem) section below for more information.

## Overview

`copyfile` enables seamless file sharing between applications by copying the file itself to the clipboard rather than just its path. This allows you to:

- Paste actual files into messaging apps, emails, and document editors
- Drag files from the clipboard into application windows
- Paste a file as if you had manually performed a copy operation in Finder

## Installation

```bash
# Using the update utility (recommended)
update ~/src/copyfile

# The update utility copies the shell wrapper script to your ~/bin directory,
# which then builds and runs the Rust program when needed
```

### Related Tools Installation

To get the full `taku` ecosystem experience, you may want to install these related tools:

```bash
# Install the update utility
git clone https://github.com/yourusername/update.git ~/src/update
cd ~/src/update
./main.sh

# Install the init utility
git clone https://github.com/yourusername/init.git ~/src/init
cd ~/src/init
./main.sh
```

## Usage

```bash
# Copy a file to clipboard
copyfile path/to/file.ext

# Copy from specific locations
copyfile ~/Documents/notes.txt
copyfile ./project/image.png
copyfile /absolute/path/to/file.pdf
```

## Scaffold From `tree` Output

You can scaffold a directory structure by pasting the output of `tree` (or a simple indented list) into the `maketree` CLI. It understands classic `tree` output (with or without `-F`) and a plain format where directories end with `/` and indentation uses two spaces per level.

Examples:

```bash
# From tree output (recommended to include slashes with -F)
tree -F myproject | maketree

# Or paste a simple indented list, then press Ctrl+D
cat <<'EOF' | maketree
app/
  src/
    main.rs
  Cargo.toml
EOF

# Dry-run to preview actions
tree -F myproject | maketree --dry-run
```

Options: `--dry-run`, `--force`, `--file <path>`, and verbosity `-v`, `-vv`, `-vvv`.

## Features

- Copies the actual file reference to clipboard, not just the path text
- Resolves relative paths automatically
- Works with all file types
- macOS-native integration using AppleScript
- Shows confirmation with the absolute path when successful

## How It Works

The `copyfile` command is a shell wrapper that:
1. Builds the Rust binary in debug mode (for faster compilation)
2. Executes the binary with your arguments

The underlying Rust program then:
- Resolves the file path to an absolute path
- Uses AppleScript's `System Events` to place a file reference directly onto the system clipboard
- This is equivalent to selecting a file in Finder and pressing Command+C

```rust
tell application "System Events"
    set the clipboard to (POSIX file "/path/to/file")
end tell
```

## Examples

### Attach a file to an email

```bash
copyfile ~/Documents/report.pdf
# Now paste into an email composer to attach the file
```

### Share a file via messaging app

```bash
copyfile ~/Pictures/screenshot.png
# Paste into Messages, Slack, Discord, etc.
```

### Insert an image into a document

```bash
copyfile ~/Desktop/diagram.jpg
# Paste into Pages, Microsoft Word, Google Docs, etc.
```

## Requirements

- macOS (relies on AppleScript for clipboard operations)
- Rust (required for building each time the command is run)
- Cargo (the Rust package manager)

## Part of the `taku` Ecosystem

This tool is part of the `taku` ecosystem - a family of CLI utilities and development tools designed to work together:

| Tool | Purpose |
|------|---------|
| `copyfile` | Copy file references to clipboard |
| `update` | Watch and sync CLI tools to ~/bin |
| `init` | Initialize project structures |
| `taku` | The core package manager (coming soon) |

The `taku` ecosystem aims to be a streamlined package manager and build tool for Rust projects (similar to how `yarn` works for JavaScript). When fully implemented, `taku` will:

- Manage Rust dependencies more efficiently
- Provide faster builds with intelligent caching
- Simplify the installation and updating of Rust CLI tools
- Offer a consistent interface for developing and distributing Rust applications
- Support development across different environments (dev, staging, prod)

These separate CLI tools are being developed as modular components that will eventually integrate with the core `taku` functionality.

## License

0BSD License
