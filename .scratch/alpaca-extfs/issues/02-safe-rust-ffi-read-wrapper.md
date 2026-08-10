# 02 — Safe Rust FFI Abstraction (`Ext2FsHandle`) for Read Operations

**Status:** done

## What to build

A safe Rust FFI module (`src/ffi.rs`) and high-level handle (`Ext2FsHandle`) wrapping `libext2fs` C API functions for opening ext4 disk devices/images, reading superblocks, reading inodes, and reading directory listings.

User stories covered: #2 (List files & metadata), #3 (Read file contents).

## Acceptance criteria

- [x] FFI module defining C bindings to `ext2fs_open`, `ext2fs_close`, `ext2fs_read_inode`, `ext2fs_dir_iterate2`, and `ext2fs_file_open`/`read`.
- [x] `Ext2FsHandle` struct providing safe Rust wrappers around raw `ext2_filsys` pointers.
- [x] Methods to list files in a directory path and query inode attributes (size, permissions, timestamps).
- [x] Unit test opening a sample Ext4 disk image and listing its root directory.

## Blocked by

- 01 — Project Scaffolding & Vendored `libext2fs` Static Compilation
