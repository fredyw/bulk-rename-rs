# Rules for AI Agents

Welcome! As an AI agent working on the `bmv` repository, please adhere to the following rules and guidelines to ensure high-quality contributions.

## 1. Testing Requirements
**Every change must be accompanied by tests.** Whether it's a new feature, a bug fix, or a refactoring effort, you must include:
- **Unit Tests**: For internal logic, regex matching, and path manipulation.
- **Integration Tests**: For CLI behavior, filesystem interactions, and end-to-end renaming flows.

### Running Tests
Before submitting any change, ensure all tests pass:
```bash
./test.sh
```

## 2. Coding Standards
- **Idiomatic Rust**: Follow standard Rust idioms and best practices.
- **Verification**: Running `./test.sh` automatically performs formatting and linting checks before running tests.
- **Fix**: Use the provided script to automatically fix formatting and common clippy lints.
  ```bash
  ./fix.sh
  ```

## 3. Documentation
- Update `README.md` if you add new CLI arguments or change existing functionality.
- Use docstrings (`///`) for public functions and structs in `src/lib.rs` and `src/bulk_rename.rs`.

## 4. Commit Messages
- **Structure**: Every commit must have a clear title and a descriptive body.
- **Content**: Explain *what* was changed and *why*. Avoid one-liner commits for non-trivial changes.
- **Atomic Commits**: Keep each commit focused on a single logical change. Do not group unrelated changes (e.g., a bug fix and a refactor) into the same commit.
- **Conciseness**: Be descriptive but avoid being overly verbose. Focus on key technical decisions or rationale.

## 5. Performance & Safety
Since `bmv` is a bulk file renaming tool:
- **Safety First**: Ensure that file operations are safe and handle potential conflicts (e.g., name collisions) gracefully.
- **Parallelism**: Leverage `rayon` for efficient parallel processing of large directories.
- **Regex Efficiency**: Be mindful of regex compilation and performance, especially when processing thousands of files.
- **Dry Run**: Always consider how changes affect the "dry run" functionality to ensure users can preview changes safely.

## 6. Repository Structure
- `src/lib.rs`: Main library entry point (module declarations).
- `src/bulk_rename.rs`: Core logic for finding and renaming files.
- `src/main.rs`: CLI entry point and argument parsing using `clap`.
- `tests/`: Integration tests for filesystem operations and CLI logic.

Thank you for contributing to `bmv`!
