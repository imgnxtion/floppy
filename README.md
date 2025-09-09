# meta

Previously `maketree` and `floppy`.

TLDR: A lightweight command-line utility for macOS that copies file contents.

First, a few terms...

This application, Meta, is designed to extract and manage metadata from files—specifically, the contents within those files, rather than their file paths. Unlike traditional methods of handling file metadata, Meta offers the ability to copy the extracted content directly to the user's clipboard. This isn't just a one-time action; you can seamlessly pass around this content and reuse it across different applications, empowering you to work more fluidly with file data.
To better understand the nature of Meta and its purpose, let us explore the origins of the term "meta" and how it has evolved in modern usage.

**Dual Definition of "Meta"**

In Greek and Latin:

**Greek**: The prefix "μετά" (`meta`) originates from Greek and means "beyond" or "after." It is used to indicate a state that goes beyond or transcends the original subject. Philosophically, it refers to the study of the essence or nature of something, such to "go beyond" a concept and examine it on a higher level. For example, "metaphysics" deals with the nature of reality, moving beyond the physical realm.

**Latin**: In Latin, "meta" means "goal" or "boundary," referring to a significant point or marker, often in a physical sense, such as the turning point in a race. Metaphorically, it signifies limits or boundaries within a process or context.

**In Modern English**:
Meta has come to represent something that is self-referential, abstract, or about the essence of itself. In modern English, "meta" often denotes a concept that refers to itself or transcends its ordinary context. When we talk about "metadata," we refer to data about data—information that describes other data. In the case of Meta, this application facilitates the process of extracting and copying content-based metadata from files, specifically focusing on the contents rather than file paths. Once extracted, the metadata is copied directly to your clipboard, allowing you to easily pass it around and use it across different platforms, applications, and workflows.

## How It Works

### Copying Files

- **Text Files**: Reads the file's **content** (ASCII text) and copies it to clipboard as text
- **Binary Files**: Uses AppleScript to copy file reference to clipboard (for pasting as the actual file)
- **Directories**: Runs `tree -F` on the directory and copies the **structure content** to clipboard - the
  "meta" information about the layout

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

````bash
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
````

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
