# ghostsync

**ghostsync** is a file synchronization tool written in Rust that is built to handle the **worst-case scenarios** of cloud storage file behavior—especially those seen on macOS with iCloud's "Optimize Storage" feature.

Where traditional tools like `rsync`, `cp`, or `tar` may hang, error, or silently skip files, `ghostsync` is designed to **detect, report, and safely handle**:
- `dataless` files that haven't been downloaded yet (using BSD flags or Apple xattrs),
- invisible metadata quirks introduced by iCloud and Finder,
- unexpected sync failures caused by cloud-managed attributes.

## Key Features

- ✅ **Detects `dataless` files** using `ls -ldO` and xattrs (safe, no `unsafe` code).
- ✅ **Prints meaningful diagnostics** when files are skipped or problematic.
- ✅ **Walks and filters files robustly**, using `walkdir` and xattr inspection.
- ✅ **Handles macOS/iCloud corner cases** better than common tools.

## Example

```sh
ghostsync ~/Documents ~/Backup
```

This will walk the source tree and copy to the destination, flagging any skipped or problematic files with explanations.

## Installation

```sh
cargo install ghostsync
```

*Requires Rust 1.70+ and macOS (due to reliance on macOS-specific tools).*

## License

MIT License © 2025 Nathan Dautenhahn
