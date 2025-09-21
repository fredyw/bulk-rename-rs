# bmv

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![bmv](https://github.com/fredyw/bmv/actions/workflows/bmv.yml/badge.svg)](https://github.com/fredyw/bmv/actions/workflows/bmv.yml)

A CLI to do a bulk rename.

## Usage

### CLI
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

### API
```rust
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

match BulkRename::new(Path::new("tmp"), r"(test)_(\d+).txt", r"${2}_${1}.txt") {
    Ok(br) => {
        br.bulk_rename(SimpleCallback::new());
    }
    Err(e) => {
        eprintln!("Error: {:?}", e);
    }
}
```
