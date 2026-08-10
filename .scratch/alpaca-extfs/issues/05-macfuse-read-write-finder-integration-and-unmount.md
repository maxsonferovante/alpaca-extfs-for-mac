# 05 — macFUSE Full Read/Write Finder Integration & Graceful Unmount

## What to build

Complete macFUSE read-write implementation supporting Finder file creation, text/binary modifications, file deletion, and directory creation. Includes `Arc<Mutex<Ext2FsHandle>>` thread synchronization and signal handling (`SIGINT`/`SIGTERM`) to guarantee clean unmounting and journal flushing.

User stories covered: #4 (Finder file creation), #5 (Finder file editing), #6 (Finder file deletion), #7 (Safe unmount & flush), #9 (Thread safety under concurrent requests).

**Status:** done

## Acceptance criteria

- [x] Implement `fuser` callbacks for `create`, `write`, `unlink`, `mkdir`, `rmdir`, `flush`, and `release`.
- [x] Wrap `Ext2FsHandle` in `Arc<Mutex<...>>` to synchronize concurrent Finder requests safely.
- [x] Trigger `ext2fs_flush` after write, unlink, or mkdir operations.
- [x] Implement signal handler catching `SIGINT`/`SIGTERM` to call `ext2fs_flush` and `fuse_unmount` cleanly.
- [x] Verify creating, modifying, and deleting files directly inside macOS Finder.

## Blocked by

- 03 — macFUSE Mount & Read-Only Directory/File Browsing in Finder
- 04 — Ext4 Write & Inode Allocation FFI (`Ext2FsHandle` Write API)
