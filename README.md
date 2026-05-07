# bulk-rename-rs

<p align="center">
  <img src="assets/logo.png" alt="bulk-rename-rs Logo" width="400">
</p>

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![CI](https://github.com/fredyw/bulk-rename-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/fredyw/bulk-rename-rs/actions/workflows/ci.yml)
[![Publish](https://github.com/fredyw/bulk-rename-rs/actions/workflows/publish.yml/badge.svg)](https://github.com/fredyw/bulk-rename-rs/actions/workflows/publish.yml)
[![Crates.io](https://img.shields.io/crates/v/bulk-rename-rs.svg)](https://crates.io/crates/bulk-rename-rs)

A powerful command-line tool for bulk renaming files using regular expressions, built with Rust for speed and safety.

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
    - [CLI Reference](#cli-reference)
    - [Library API](#library-api)
- [Features](#features)
    - [Dynamic Variables](#dynamic-variables)
    - [Text Transformations](#text-transformations)
    - [Collision Handling](#collision-handling)
    - [Transactional Renames](#transactional-renames)
    - [Filtering & Modes](#filtering--modes)
- [Development](#development)
    - [Building](#building)
    - [Testing](#testing)
    - [Releasing](#releasing)
- [Contributing](#contributing)
- [License](#license)

---

## Installation

### From Prebuilt Binaries

**Linux & macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/fredyw/bulk-rename-rs/main/install.sh | bash
```

**Windows (PowerShell):**

```powershell
iwr -useb https://raw.githubusercontent.com/fredyw/bulk-rename-rs/main/install.ps1 | iex
```

### From crates.io

If you have Rust installed, you can install `brnm` directly from [crates.io](https://crates.io/crates/bulk-rename-rs):

```bash
cargo install bulk-rename-rs
```

### From Source

```bash
git clone https://github.com/fredyw/bulk-rename-rs.git
cd bulk-rename-rs
./install.sh --source
```

---

## Quick Start

Get started with common renaming tasks:

### Basic Regex Rename
Rename all `.txt` files by adding a prefix:
```bash
brnm -f . -r "(.*)\.txt" -p "prefix_$1.txt"
```

### Add a Sequential Counter
Rename files to `image_001.jpg`, `image_002.jpg`, etc.:
```bash
brnm -f ./photos -r ".*\.jpg" -p "image_{i:3}.jpg"
```

### Case Transformation
Convert all filenames to uppercase:
```bash
brnm -f . -r "(.*)" -p "{u:$1}"
```

### Dry Run (Safety First)
Preview changes without applying them:
```bash
brnm -f . -r "old" -p "new" --dry-run
```

---

## Usage

### CLI Reference

```bash
Usage: brnm [OPTIONS] --dir <DIR>
       brnm [OPTIONS] --dir <DIR> --regex <REGEX> --replacement <REPLACEMENT>
       brnm --undo [OPTIONS]

Options:
  -f, --dir <DIR>                  Set the directory
  -r, --regex <REGEX>              Set the regex (required unless --undo is present)
  -p, --replacement <REPLACEMENT>  Set the replacement (required unless --undo is present)
  -d, --dry-run                    Perform a dry-run
  -q, --quiet                      Run in quiet mode
  -i, --interactive                Prompt for confirmation before each rename
  -I, --ignore-case                Use case-insensitive matching
  -e, --ext <EXT>                  Filter files by extension (comma-separated)
      --include <INCLUDE>          Include only files matching these patterns (comma-separated)
      --exclude <EXCLUDE>          Exclude files matching these patterns (comma-separated)
      --max-depth <MAX_DEPTH>      Set the maximum depth for recursion (1 for current directory only)
  -c, --collision <STRATEGY>       Set the collision strategy [default: skip] [possible values: skip, overwrite, suffix]
      --undo                       Undo the previous rename operation
      --history-file <PATH>        Set the history file path [default: .brnm-undo.json]
      --counter-start <START>      Set the starting value for the counter {i} [default: 1]
  -m, --mode <MODE>                Set the renaming mode [default: files] [possible values: files, dirs, all]
  -s, --symlinks <STRATEGY>        Set the symlink strategy [default: ignore] [possible values: ignore, rename, follow]
  -T, --transaction <STRATEGY>     Set the transaction strategy [default: continue] [possible values: continue, abort, rollback]
  -h, --help                       Print help
  -V, --version                    Print version
```

### Library API

`bulk-rename-rs` can be integrated into your Rust projects as a library:

```rust
use bulk_rename_rs::{BulkRename, Callback, NoOpCallback};
use std::path::Path;

fn main() {
    let bulk_rename = BulkRename::new(
        Path::new("./files"), 
        r"old_(.*)\.txt", 
        r"new_$1.txt"
    ).unwrap();
    
    // Execute renames in parallel
    bulk_rename.execute(NoOpCallback::new());
}
```

---

## Features

### Dynamic Variables
Inject dynamic metadata into your filenames:
- `{i}`: Auto-incrementing counter.
- `{i:N}`: Counter with zero-padding (e.g., `{i:3}` -> `001`).
- `{date}`: File modification date (`YYYY-MM-DD`).
- `{date:FORMAT}`: Custom date format (e.g., `{date:%Y%m%d}`).

**Example:**
```bash
# Rename to: 2023-10-27_001.log
brnm -f . -r ".*\.log" -p "{date}_{i:3}.log"
```

### Text Transformations
Apply transformations to capture groups or static text:
- `{u:TEXT}` / `{upper:TEXT}`: UPPERCASE
- `{l:TEXT}` / `{lower:TEXT}`: lowercase
- `{t:TEXT}` / `{title:TEXT}`: Title Case

**Example:**
```bash
# Matches "report_final.doc" -> "REPORT_Final.doc"
brnm -f . -r "(.*)_(.*)\.doc" -p "{u:$1}_{t:$2}.doc"
```

### Collision Handling
Control what happens when a target filename already exists:
- `skip` (default): Skip the file.
- `overwrite`: Overwrite the existing file.
- `suffix`: Append a numeric suffix (e.g., `file (1).txt`).

### Transactional Renames
Ensure consistency during bulk operations:
- `continue` (default): Skip errors and keep going.
- `abort`: Stop on the first error.
- `rollback`: Stop and undo all successful renames from the current session if an error occurs.

### Filtering & Modes
- **File Types**: Filter by extension (`--ext jpg,png`).
- **Path Filtering**: Use `--include` and `--exclude` regex patterns.
- **Recursion**: Control depth with `--max-depth`.
- **Modes**: Rename `files`, `dirs`, or `all`.
- **Symlinks**: Choose to `ignore`, `rename` the link, or `follow` to the target.

---

## Development

### Building

To build the project, you need to have Rust installed. You can install it from [here](https://www.rust-lang.org/tools/install).

Once you have Rust installed, you can build the project by running the following command:

```bash
./build.sh --release
```

The binary will be located in `target/release/brnm`.

### Testing

To run the tests, including formatting and linting checks, you can use the following command:

```bash
./test.sh
```

### Releasing

To create a new release, use the provided `release.sh` script:

```bash
./release.sh <version>
```

Example:
```bash
./release.sh 0.1.0
```

---

## Contributing

Contributions are welcome! 

### AI Agents
If you are an **AI Agent** contributing to this repository, please read **[AGENTS.md](AGENTS.md)** before making any changes. It contains specific rules and workflows designed for agentic contributions.

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.
