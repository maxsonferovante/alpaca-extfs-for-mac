# 04 — Ext4 Write & Inode Allocation FFI (`Ext2FsHandle` Write API)

## What to build

Expansion of the Rust FFI bindings and `Ext2FsHandle` wrapper to support write operations: creating new inodes, linking directory entries, writing data blocks, unlinking/deleting files, creating directories, and flushing filesystem state (`ext2fs_flush`).

User stories covered: #4 (Creating files/folders), #5 (Editing files), #6 (Deleting files).

## Acceptance criteria

- [ ] Add FFI bindings for `ext2fs_new_inode`, `ext2fs_link`, `ext2fs_unlink`, `ext2fs_mkdir`, `ext2fs_file_write`, `ext2fs_write_inode`, and `ext2fs_flush`.
- [ ] Implement safe Rust methods on `Ext2FsHandle` for creating files, writing bytes, deleting files, and creating directories.
- [ ] Implement compulsory `flush()` method ensuring block allocation and inode bitmaps are committed to disk.
- [ ] Unit tests verifying file creation and write persistence on test disk images.

## Blocked by

- 02 — Safe Rust FFI Abstraction (`Ext2FsHandle`) for Read Operations
