# 03 — macFUSE Mount & Read-Only Directory/File Browsing in Finder

## What to build

CLI entrypoint `mount_ext4` integrating `fuser` (macFUSE). Mounting an Ext4 device path or image file onto a macOS folder allows the user to open Finder, browse directory trees, inspect file sizes/dates, and open/read text and binary files in read-only mode.

User stories covered: #1 (Mount via `mount -t ext4`), #2 (Finder browsing), #3 (Reading files in Finder).

## Acceptance criteria

- [ ] Implement `fuser::Filesystem` trait callbacks for `lookup`, `getattr`, `readdir`, `open`, and `read`.
- [ ] Implement CLI command parsing (`mount_ext4 <device> <mount_point>`) validating root/sudo execution and raw disk device path (`/dev/rdisk`).
- [ ] Mount Ext4 image/device to macOS mount point using macFUSE.
- [ ] Finder displays directory contents and reads file content cleanly.

## Blocked by

- 02 — Safe Rust FFI Abstraction (`Ext2FsHandle`) for Read Operations
