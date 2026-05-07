# bmv

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![CI](https://github.com/fredyw/bmv/actions/workflows/ci.yml/badge.svg)](https://github.com/fredyw/bmv/actions/workflows/ci.yml)

A powerful command-line tool for bulk renaming files using regular expressions, built with Rust for speed and safety.

## Table of Contents

- [Features](#features)
    - [Dynamic Variables](#dynamic-variables)
    - [Text Transformations](#text-transformations)
    - [Collision Handling](#collision-handling)
    - [Undo & Rollback](#undo--rollback)
    - [Filtering](#filtering)
    - [Interactive Mode](#interactive-mode)
- [Usage](#usage)
    - [CLI](#cli)
    - [API](#api)
- [Installation](#installation)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

## Features

### Dynamic Variables
Placeholders in the replacement string allow for dynamic naming:
- `{i}`: An auto-incrementing counter. Use `{i:N}` (e.g., `{i:3}`) for padding (e.g., `001`).
- `{date}`: The file's modification date in `%Y-%m-%d` format.
- `{date:FORMAT}`: The file's modification date with a custom [chrono format](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) (e.g., `{date:%Y%m%d}`).

### Text Transformations
Apply built-in transformations to matched groups or static text:
- `{u:TEXT}` or `{upper:TEXT}`: Convert to UPPERCASE.
- `{l:TEXT}` or `{lower:TEXT}`: Convert to lowercase.
- `{t:TEXT}` or `{title:TEXT}`: Convert to Title Case.

Example: `-p "{u:$1}_{l:$2}_{t:$1 $2}.txt"`

> [!NOTE]
> When dynamic variables are used, files are processed in alphabetical order to ensure deterministic assignment.

### Collision Handling
Define how to handle cases where the target filename already exists using the `--collision` flag:
- `skip` (default): Skip the rename if the destination exists.
- `overwrite`: Replace the existing file.
- `suffix`: Append a numeric suffix (e.g., `file.txt` -> `file (1).txt`).

### Undo & Rollback
Mistakes happen. `bmv` tracks renames in a history file (defaults to `.bmv-undo.json`), allowing you to revert the last operation:
```bash
bmv --undo
```

### Filtering
Precisely target files using multiple filtering options:
- **Extensions**: Filter by file extension (e.g., `--ext jpg,png`).
- **Include/Exclude**: Use regex patterns to include or exclude specific files.
- **Max Depth**: Control recursion depth (e.g., `--max-depth 1` for current directory only).
- **Renaming Mode**: Specify whether to rename files, directories, or both using the `--mode` (or `-m`) flag.

> [!IMPORTANT]
> **Precedence**: `exclude` patterns have the highest priority. If a file matches both an `include` and an `exclude` pattern, it will be **excluded**.

### Interactive Mode
For sensitive renames, use the `--interactive` (or `-i`) flag to prompt for confirmation before each file is renamed.

### Parallel Execution
`bmv` leverages `rayon` to perform renaming operations in parallel across multiple threads, making it extremely fast even for thousands of files.

## Usage

### CLI

```bash
Usage: bmv [OPTIONS] --dir <DIR>
       bmv [OPTIONS] --dir <DIR> --regex <REGEX> --replacement <REPLACEMENT>
       bmv --undo [OPTIONS]

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
      --history-file <PATH>        Set the history file path [default: .bmv-undo.json]
      --counter-start <START>      Set the starting value for the counter {i} [default: 1]
  -m, --mode <MODE>                Set the renaming mode [default: files] [possible values: files, dirs, all]
  -h, --help                       Print help
  -V, --version                    Print version
```

### API

`bmv` can also be used as a library in your Rust projects:

```rust
use bmv::{BulkRename, Callback, CollisionStrategy};
use std::path::Path;

struct SimpleCallback {}

impl Callback for SimpleCallback {
    fn on_ok(&self, old_path: &Path, new_path: &Path) {
        println!("OK: {} --> {}", old_path.display(), new_path.display());
    }

    fn on_error(&self, old_path: &Path, new_path: &Path, error: std::io::Error) {
        eprintln!("Error: Unable to rename {} to {}: {}", old_path.display(), new_path.display(), error);
    }
}

fn main() {
    let bulk_rename = BulkRename::new(Path::new("./files"), r"old_(.*)\.txt", r"new_$1.txt").unwrap();
    
    // Execute renames in parallel with a callback
    bulk_rename.execute(SimpleCallback::new());
}
```

## Installation

To install `bmv`, you can use the provided installation script:

```bash
./install.sh
```

## Testing

To run the test suite, use the following command:

```bash
./test.sh
```

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.

## License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.
