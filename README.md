# bmv

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![CI](https://github.com/fredyw/bmv/actions/workflows/ci.yml/badge.svg)](https://github.com/fredyw/bmv/actions/workflows/ci.yml)

A CLI to do a bulk rename.

## Table of Contents

- [Usage](#usage)
    - [CLI](#cli)
    - [API](#api)
- [Building](#building)
- [Installing](#installing)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

### Usage

#### CLI

```
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
  -h, --help                       Print help
  -V, --version                    Print version

> [!NOTE]
> **Dynamic Variables:** You can use `{i}` for an auto-incrementing counter in the replacement string. 
> Use `{i:N}` (e.g., `{i:3}`) to specify padding with leading zeros (e.g., `001`, `002`).
> **Note:** When `{i}` is used, files are processed in alphabetical order to ensure deterministic counter assignment.

> [!NOTE]
> **Precedence:** `exclude` patterns have the highest priority. If a file matches both an `include` and an `exclude` pattern, it will be **excluded**.
```

#### API

```rust
use bmv::{BulkRename, Callback, CollisionStrategy};
use std::path::Path;

struct SimpleCallback {}

impl SimpleCallback {
    fn new() -> Self {
        Self {}
    }
}

impl Callback for SimpleCallback {
    fn on_ok(&self, old_path: &Path, new_path: &Path) {
        println!("OK: {} --> {}", old_path.display(), new_path.display());
    }

    fn on_error(&self, old_path: &Path, new_path: &Path, error: std::io::Error) {
        eprintln!(
            "Error: Unable to rename {} to {}: {}",
            old_path.display(),
            new_path.display(),
            error
        );
    }
}

let dir = Path::new("tmp");
match BulkRename::new(dir, r"(test)_(\d+).txt", r"${2}_${1}.txt") {
    Ok(br) => {
        let br = br.with_collision_strategy(CollisionStrategy::Suffix);
        br.bulk_rename(SimpleCallback::new());
    }
    Err(e) => {
        eprintln!("Error: {:?}", e);
    }
}
```

### Installing

To install `bmv`, you can use the following command.

```
./install.sh
```

### Testing

To run the tests, you can use the following command.

```
./test.sh
```

### Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.

### License

This project is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.
