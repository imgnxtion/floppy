# meta

A lightweight command-line utility for macOS that copies file contents (ASCI## How It Works

### Copying Files
- **Text Files**: Reads the file's **content** (ASCII text) and copies it to clipboard as text
- **Binary Files**: Uses AppleScript to copy file reference to clipboard (for pasting as the actual file)
- **Directories**: Runs `tree -F` on the directory and copies the **structure content** to clipboard - the "meta" information about the layout

### Pasting
- Detects if clipboard contains tree structure or file content
- Recreates directories/files accordinglyamples

### Copy text file content

```bash
meta ~/scripts/build.sh
# The content of build.sh is now in clipboard
```

### Copy binary file for pasting

```bash
meta ~/images/logo.png
# File reference copied - paste into X.com, email, etc. as the actual file
```

### Copy directory structure content (meta)

```bash
meta ~/myapp
# Directory structure content (tree output) copied to clipboard - the "meta" of the layout
```

### Paste directory structure

```bash
meta -p ~/newapp
# Recreates the directory structure from clipboard
```

### Paste file content

```bash
echo "Hello World" | pbcopy
meta -p ~/hello.txt
# Creates hello.txt with "Hello World" content
``` or directory **structure content** (the "meta" layout) to/from the clipboard. For binary files, it copies the file reference for pasting as a file.

> **Note:** This tool is designed to be lightweight and only handles ASCII text file contents and directory structure metadata. Binary files are copied as file references to maintain compatibility with applications like X.com, messaging apps, etc.

## Overview

`meta` enables seamless copying and pasting of file **contents** or directory **structure content** (the "meta" layout) via the clipboard:

- **Text Files**: Copies the file's **content** (ASCII text) to clipboard
- **Binary Files**: Copies the file reference for pasting as the actual file
- **Directories**: Copies the directory **structure content** (tree output) to clipboard - the "meta" of the layout
- **Paste**: Recreates files or directory structures from clipboard contents

## Installation

```bash
# Build the project
cargo build --release

# Copy the binary to your PATH
cp target/release/meta ~/bin/
```

## Usage

```bash
# Copy a text file's content to clipboard
meta ~/Documents/notes.txt

# Copy a binary file reference to clipboard (for pasting as file)
meta ~/Pictures/screenshot.png

# Copy directory structure content (meta) to clipboard
meta ~/myproject

# Paste from clipboard
meta -p ~/destination

# Get help
meta --help
```

## Features

- **Lightweight**: Only copies ASCII text contents for text files
- **Smart Detection**: Automatically detects text vs binary files
- **Directory Support**: Copies and recreates directory structures
- **Clipboard Integration**: Uses macOS native clipboard
- **Cross-Application**: Works with any app that supports clipboard operations

## How It Works

### Copying Files
- **Text Files**: Reads the file's **content** (ASCII text) and copies it to clipboard as text
- **Binary Files**: Uses AppleScript to copy file reference to clipboard (for pasting as the actual file)
- **Directories**: Runs `tree -F` on the directory and copies the output to clipboard

### Pasting
- Detects if clipboard contains tree structure or file content
- Recreates directories/files accordingly

## Examples

### Copy text file content

```bash
meta ~/scripts/build.sh
# The content of build.sh is now in clipboard
```

### Copy binary file for pasting

```bash
meta ~/images/logo.png
# File reference copied - paste into X.com, email, etc. as the actual file
```

### Copy directory structure

```bash
meta ~/myapp
# Tree output (structure) copied to clipboard
```

### Paste directory structure

```bash
meta -p ~/newapp
# Recreates the directory structure from clipboard
```

### Paste file content

```bash
echo "Hello World" | pbcopy
meta -p ~/hello.txt
# Creates hello.txt with "Hello World" content
```

## Requirements

- macOS (uses AppleScript and pbcopy/pbpaste)
- Rust (for building)
- `tree` command (for directory structure copying)

## License

0BSD License
