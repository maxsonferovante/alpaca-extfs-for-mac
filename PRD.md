# Product Requirement Document (PRD) - Driver Ext4 Read/Write para macOS (`mount_ext4`)

## Problem Statement

Users on macOS who need to read from and write to Linux Ext4-formatted storage devices (such as external hard drives, USB flash drives, or disk image files) currently lack a native, high-performance, write-capable filesystem driver integrated with Finder. Existing free tools are either read-only or unstable, while proprietary options are closed-source and paid. macOS users need a reliable, open-source solution that allows seamless browsing, editing, creating, and deleting files on Ext4 volumes directly within Finder and macOS terminal tools.

---

## Solution

Build a user-space Ext4 filesystem driver (`mount_ext4`) for macOS written in **Rust**. The driver bridges macOS Finder file operations to the official Linux **`libext2fs`** library (from `e2fsprogs`, statically vendored and compiled) using **`macFUSE`**. The tool is exposed as a helper binary compatible with native `mount -t ext4` syntax and handles full read/write operations safely with thread synchronization and automatic journal/bitmap flushing.

---

## User Stories

1. As a macOS user with a Linux dual-boot or external Ext4 disk, I want to mount my Ext4 volume via command line (`sudo mount -t ext4 /dev/rdisk4s2 /Volumes/Ext4Drive`), so that I can access my Linux files natively on Mac.
2. As a macOS user browsing an Ext4 drive in Finder, I want to view directories, subfolders, and file metadata (size, permissions, timestamps), so that I can inspect my files effortlessly.
3. As a macOS user, I want to open and read existing files stored on an Ext4 drive without file corruption or performance degradation.
4. As a macOS user, I want to create new files and folders directly from Finder or terminal on an Ext4 drive, so that I can transfer data from macOS to Linux drives.
5. As a macOS user, I want to edit and save existing text/binary files on an Ext4 volume, so that changes are written accurately to disk.
6. As a macOS user, I want to delete files and folders from an Ext4 drive, so that inode blocks and allocation bitmaps are freed properly.
7. As a macOS user, I want to safely unmount the Ext4 drive (`umount /Volumes/Ext4Drive`), so that all cached blocks and ext4 journal state are flushed to disk before disconnecting.
8. As a developer, I want the driver to build without needing Homebrew runtime dynamic libraries, so that the executable is self-contained.
9. As a system administrator, I want concurrent read/write operations from macOS applications to be thread-safe, so that multi-threaded I/O doesn't corrupt the Ext4 inode table.

---

## Implementation Decisions

### Architectural Topology
- **User-Space FS Engine**: `macFUSE` via the Rust `fuser` crate.
- **Ext4 Parser Engine**: Linux `libext2fs` (from `e2fsprogs`), compiled as a static C library via `build.rs` and the `cc` crate.
- **FFI Layer**: Safe Rust abstractions (`Ext2FsHandle`) wrapping `libext2fs` C API functions (`ext2fs_open`, `ext2fs_read_inode`, `ext2fs_file_read`, `ext2fs_file_write`, `ext2fs_new_inode`, `ext2fs_link`, `ext2fs_unlink`, `ext2fs_mkdir`, `ext2fs_flush`, `ext2fs_close`).

### Concurrency & Data Safety
- Thread synchronization using `Arc<Mutex<Ext2FsHandle>>`. All FUSE callback operations lock this single mutex before executing `libext2fs` operations.
- Explicit `ext2fs_flush` executed after file creation, write, or deletion operations to prevent data loss on sudden unmounts.

### Device Access & Privileges
- Drive mounting requires root/elevated privileges (`sudo`).
- Reads/writes target the raw character disk device `/dev/rdiskXsY` (or disk image file paths) to bypass macOS kernel double-buffering.

### CLI UX
- Binary name: `mount_ext4`.
- Accepts standard macOS `mount` syntax: `mount_ext4 [-o ro|rw] <device_path> <mount_point>`.
- Registers signal handlers (`SIGINT`, `SIGTERM`, `SIGHUP`) to unmount macFUSE cleanly and close the `ext2_filsys` handle.

---

## Testing Decisions

### Test Strategy
- Tests must verify external system behavior (mounting, directory listing, read accuracy, write persistence, unmount safety, `e2fsck` integrity check) rather than internal FFI memory layouts.

### Modules Tested
- **`Ext2FsHandle` (Rust Wrapper Module)**: Tested against loopback Ext4 disk image files (`.img`).
- **`mount_ext4` CLI & FUSE callbacks**: End-to-end mounting, file creation, writing, reading back, deleting, unmounting, and verifying disk integrity with `e2fsck`.

### Prior Art & Test Setup
- Automated creation of temporary 64MB Ext4 images using `dd` and `mkfs.ext4` during integration testing.

---

## Out of Scope

- Graphical User Interface (GUI) desktop application or menu bar status app.
- Partition management or formatting utility (creating new ext4 filesystems; `mkfs.ext4` remains separate).
- Apple FSKit / Kernel Extension driver development (macFUSE is used exclusively).
- Extended ACLs or Linux SELinux security context translation.

---

## Further Notes

- `macFUSE` installation on macOS requires kernel extension approval or macFUSE FUSE library installed on the host system.
- Dirty ext4 journals (`EXT3_FEATURE_INCOMPAT_RECOVER`) will be auto-recovered or opened with `EXT2_FLAG_FORCE` if cleanly unmounted flag is missing.
