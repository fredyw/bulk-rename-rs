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
Usage: bmv [OPTIONS] --dir <DIR> --regex <REGEX> --replacement <REPLACEMENT>

Options:
  -f, --dir <DIR>                  Set the directory
  -r, --regex <REGEX>              Set the regex
  -p, --replacement <REPLACEMENT>  Set the replacement
  -d, --dry-run                    Perform a dry-run
  -q, --quiet                      Run in quiet mode
  -h, --help                       Print help
  -V, --version                    Print version
```

#### API

```rust
use bmv::{BulkRename, Callback};
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
