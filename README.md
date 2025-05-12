# anydir

A Rust library that provides a unified interface for working with both compile-time embedded directories and runtime directories.


## Motivation

It frustrated me that no Rust library had an abstracted `Dir` type that allowed for both embedded compile-time directories and runtime directories, so I made one.

If you know of such a library, please let me know !

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
anydir = "0.1.0"
```

## Usage

```rust
use anydir::anydir;

fn main() {
    // Embed files at compile time - bundled directly in your binary
    let ct_dir = anydir!(ct, "$CARGO_MANIFEST_DIR");
    let ct_files = ct_dir.list_files();
    println!("Compile-time files: {:?}", ct_files);
    
    // Access files at runtime from the filesystem
    let rt_dir = anydir!(rt, "./");
    let rt_files = rt_dir.list_files();
    println!("Runtime files: {:?}", rt_files);
}
```

## Features

- **Compile-time embedding**: Bundle directory contents directly in your binary
- **Runtime access**: Load directories from the filesystem at runtime
- **Unified API**: Work with both types using the same interface
- **Zero-cost abstraction**: No runtime overhead for compile-time directories
- **Simple macros**: Easy-to-use macro interface

## API

### Macros

- `anydir!(ct, path)` - Create a compile-time embedded directory
- `anydir!(rt, path)` - Create a runtime directory reference

### Traits

- `DirOps` - Common operations for directories
  - `list_files()` - Returns a vector of filenames in the directory
