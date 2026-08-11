# Changelog

All notable changes to the `alpaca-extfs` project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0] - 2026-08-10

### Added
- **Background Daemon Execution**: Default CLI execution mode now detaches into a background daemon process, instantly releasing the terminal prompt upon successful volume mount.
- **Finder Auto-Launch**: Automatically opens the mounted volume location in Finder (`open /Volumes/...`) upon session initialization.
- **Non-Root Permission Mapping**: Automatically detects `SUDO_USER`, `SUDO_UID`, and `SUDO_GID` to map mounted Ext4 file/directory attributes, enabling seamless read/write access in Finder and user terminal sessions without `Permission denied` errors.
- **Unmount CLI Flag (`-u` / `--unmount`)**: Direct unmounting helper flag (`sudo alpaca-extfs -u /Volumes/Ext4Drive`).
- **Foreground Debugging Flag (`-f` / `--foreground`)**: Preserves foreground execution mode for troubleshooting.
- **FUSE Handlers**: Implemented `access`, `getxattr`, and `listxattr` handlers required by macOS LaunchServices and Finder.

### Fixed
- **Root Inode Translation**: Added bidirectional mapping between FUSE root inode (`1`) and Ext4 Linux root inode (`2`), fixing empty directory listing issues when opening mounted drives.
- **Non-UTF8 Filenames**: Handled invalid/non-UTF-8 directory entry names safely using `String::from_utf8_lossy`.
- **Directory Mode Mask**: Guaranteed directory bitmask fallback (`0o755`) so Ext4 root and subdirectories are readable and browseable by non-root users.
- **`fuse_mount_compat25` Bypass**: Fixed macFUSE 4/5 mounting errors (`Unspecified Error`) on macOS ARM64/x86_64 by binding native `fuse_mount` and `fuse_chan_fd` directly in FFI.
- **macOS `allow_other` Sysctl**: Automatically sets `vfs.generic.macfuse.tunables.allow_other=1` in kernel sysctl upon execution.

---

## [0.1.0] - 2026-08-09

### Added
- Initial release of `alpaca-extfs` Ext4 read/write FUSE driver for macOS.
- Direct `libext2fs` (`e2fsprogs` 1.47.4) static C FFI binding.
- Basic FUSE operations: `getattr`, `lookup`, `readdir`, `read`, `write`, `create`, `mkdir`, `unlink`, `rmdir`, `statfs`.
