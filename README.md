# mount_ext4 — Ext4 Read/Write Driver for macOS

A user-space Ext4 filesystem driver for macOS with full **Read & Write** support natively integrated into Finder, built with **Rust**, **macFUSE**, and Linux's official **`libext2fs`** (`e2fsprogs`).

---

## 🚀 Features

- **Full Read/Write Support**: Open, edit, create, and delete files directly in Finder or terminal.
- **Finder Native Integration**: Browse folders, inspect file sizes, modification dates, and permissions via POSIX macFUSE.
- **Self-Contained Binary**: `libext2fs` C source code is vendored and compiled statically, requiring zero Homebrew dynamic library dependencies at runtime.
- **Data Integrity & Journal Flushing**: Concurrency safety with single-mutex lock (`Arc<Mutex<Ext2FsHandle>>`) and automatic `ext2fs_flush` on write operations.
- **Native macOS Mount Syntax**: Compatible with `mount -t ext4` or direct `mount_ext4` CLI invocation.

---

## 📋 Prerequisites

1. **macOS 12+**
2. **macFUSE**: Installed on macOS (`brew install macfuse` or download from [macfuse.github.io](https://macfuse.github.io)).
3. **Rust & Cargo**: Standard Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`).

---

## 🛠️ Building

```bash
cargo build --release
```
The compiled self-contained binary will be placed at `./target/release/mount_ext4`.

---

## 💻 Usage

### Mounting an Ext4 Disk Partition or Image

Mount an Ext4 partition (e.g. `/dev/rdisk4s2`) or a disk image file to a mount point directory:

```bash
sudo ./target/release/mount_ext4 /dev/rdisk4s2 /Volumes/Ext4Drive
```

Or mount in Read-Only mode:

```bash
sudo ./target/release/mount_ext4 --read-only /dev/rdisk4s2 /Volumes/Ext4Drive
```

### Unmounting Safely

To safely unmount the drive and flush all pending Ext4 block/journal changes:

```bash
sudo umount /Volumes/Ext4Drive
```

---

## 🧪 Running Integration Tests

The project includes an end-to-end integration test that creates a 16MB loopback Ext4 disk image, performs directory creation, file creation, read/write verification, and validates disk sanity using `e2fsck`:

```bash
cargo test
```

---

## 📐 Architecture

```
[ macOS Finder / VFS ]
       │
       ▼
 [ macFUSE Kernel Extension / fuser ]
       │
       ▼
 [ mount_ext4 (Rust Driver) ]
       │
       ▼
 [ libext2fs (Statically linked e2fsprogs C archive) ]
       │
       ▼
 [ /dev/rdiskXsY (Raw character block device) ]
```
