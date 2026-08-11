# Specification (SPEC) - Driver Ext4 Read/Write para macOS (`mount_ext4`)

## Problem Statement

Users operating macOS who use Linux systems or external storage formatted as Ext4 cannot natively mount, read, write, create, or delete files directly through macOS Finder or the terminal. macOS lacks native kernel support for Ext4, and standard POSIX commands (`mount`, `ls`, `cp`, `rm`) do not function on raw Ext4 block devices without a dedicated driver.

---

## Solution

A user-space Ext4 filesystem driver (`mount_ext4`) written in **Rust** that bridges macOS Finder and system VFS calls to the official Linux **`libext2fs`** library via **`macFUSE`**. The driver handles block allocation, inode management, directory iteration, file I/O, and journal flushing in a memory-safe, thread-synchronized environment.

---

## User Stories

1. As a macOS user, I want to execute `sudo mount -t ext4 /dev/rdisk4s2 /Volumes/Ext4Drive` to mount an Ext4 partition seamlessly into macOS.
2. As a macOS user, I want Finder to display Ext4 volume contents with correct directory hierarchies, file names, file sizes, and modification dates.
3. As a macOS user, I want to open, view, and stream text and binary files stored on an Ext4 drive.
4. As a macOS user, I want to copy files from macOS into an Ext4 drive, triggering block allocation and inode creation.
5. As a macOS user, I want to modify existing files on an Ext4 volume and save changes reliably.
6. As a macOS user, I want to delete files and folders from an Ext4 drive, freeing associated Ext4 blocks and inodes.
7. As a macOS user, I want to unmount the filesystem via `umount /Volumes/Ext4Drive`, ensuring all unwritten buffers are flushed to disk before device removal.
8. As a developer, I want the project to compile `libext2fs` statically via `build.rs` so the Rust binary has no runtime Homebrew dynamic library dependencies.
9. As a system administrator, I want concurrent Finder/system requests to be synchronized through a single mutex lock to prevent disk block allocation race conditions.

---

## Implementation Decisions

### Component Architecture
- **macFUSE User-space Bridge**: Implemented using the `fuser` Rust crate. Converts Finder VFS calls (`lookup`, `getattr`, `readdir`, `open`, `read`, `write`, `create`, `unlink`, `mkdir`, `rmdir`, `flush`) into `libext2fs` function calls.
- **Vendored C Engine**: `libext2fs` from Linux `e2fsprogs` vendored inside the project repository and compiled statically using Cargo's `build.rs` and the `cc` crate.
- **FFI Bindings**: Low-level Rust FFI module auto-generated or defined to interface with `libext2fs` C symbols (`ext2fs_open`, `ext2fs_read_inode`, `ext2fs_write_inode`, `ext2fs_dir_iterate2`, `ext2fs_file_open`, `ext2fs_file_read`, `ext2fs_file_write`, `ext2fs_new_inode`, `ext2fs_link`, `ext2fs_unlink`, `ext2fs_mkdir`, `ext2fs_flush`, `ext2fs_close`).
- **Thread Synchronization Layer**: Safe wrapper `Ext2FsHandle` protected by `Arc<Mutex<...>>`. Ensures single-threaded execution across all FUSE operations to maintain Ext4 inode/bitmap integrity.

### Data Protection & Journaling
- Force disk flush (`ext2fs_flush`) after every file modification, creation, or deletion.
- Open ext4 device with `EXT2_FLAG_RW` and `EXT2_FLAG_FORCE` to handle non-cleanly unmounted dirty flags safely.

### Operating Mode
- Requires elevated root/sudo privileges to access raw block character devices (`/dev/rdiskXsY`).

---

## Testing Decisions

### Seam Strategy
- Single high-level seam: Testing `mount_ext4` external filesystem behavior against loopback Ext4 disk image files (`.img`) generated dynamically via `dd` and `mkfs.ext4`.

### Verification Criteria
1. **Mounting**: Successful mount without kernel errors or panics.
2. **Read Integrity**: File content byte matching against original test vectors.
3. **Write & Creation Integrity**: Verification that created/modified files survive unmount and re-mount cycles.
4. **Filesystem Sanity**: Running `e2fsck -f` on the raw disk image post-unmount to confirm zero inode corruption or orphaned blocks.

---

## Out of Scope

- FSKit driver implementation (focus is on macFUSE).
- GUI desktop control panel or status bar app.
- Partition creation / filesystem formatting (`mkfs.ext4`).
- Advanced Linux SELinux security contexts or extended attributes (xattrs).

---

## Further Notes

- Target environment: macOS 12+ with macFUSE installed.
- Performance optimization: Single-mutex lock prioritizes ext4 structural safety over parallel write throughput.
