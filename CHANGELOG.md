# Changelog

All notable changes to the **alpaca-extfs** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-08-10

### 🚀 Added
- **Native Ext4 Read/Write Driver (`alpaca-extfs`)**: User-space Ext4 filesystem driver for macOS with native Finder integration via `macFUSE`.
- **Safe Rust Wrapper (`Ext2FsHandle`)**: Encapsulates `libext2fs` C API functions for opening ext4 partitions/images, reading/writing inodes, allocating blocks, directory iteration, file streaming, and metadata inspection.
- **Full Write Capabilities**:
  - Inode allocation (`create_file`) and directory linking (`ext2fs_link`).
  - Block writing (`write_file`) with explicit offset support.
  - File/Directory deletion (`unlink`, `rmdir`).
  - Directory creation (`mkdir`).
  - Explicit bitmap and journal buffer flushing (`ext2fs_flush`).
- **macFUSE Integration**: Implemented POSIX `fuser::Filesystem` callbacks (`getattr`, `lookup`, `readdir`, `read`, `create`, `write`, `unlink`, `mkdir`, `rmdir`, `flush`).
- **Single-Mutex Concurrency Control**: Thread-safe execution using `Arc<Mutex<Ext2FsHandle>>` preventing ext4 metadata allocation race conditions from concurrent Finder requests.
- **Git Submodule Integration**: Added official Linux Kernel `e2fsprogs` repository (`https://github.com/tytso/e2fsprogs`) as a Git Submodule pinned to tag `v1.47.4`.
- **Hermetic & Deterministic Build (`build.rs`)**: Statically links vendored `libext2fs.a`, `libcom_err.a`, `libe2p.a`, and `libuuid.a` archives for zero Homebrew runtime `.dylib` dependencies.
- **Automated Integration Test Suite**: End-to-end integration test creating a 16MB loopback Ext4 disk image via `mke2fs`, executing file/folder operations with `alpaca-extfs`, and validating raw disk sanity post-unmount with `e2fsck -fn` (100% clean check).

### 🛠️ Changed
- Renamed project package, binary CLI name, and crate module from `mount_ext4` to **`alpaca-extfs`**.

---

## [Unreleased]
- Support for Ext4 extended file attributes (`xattr`).
- GUI status bar mount manager for macOS desktop.
