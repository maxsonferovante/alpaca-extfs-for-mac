# alpaca-extfs — Ext4 Read/Write Driver for macOS

A high-performance user-space Ext4 filesystem driver for macOS with full **Read & Write** support natively integrated into Finder, built with **Rust**, **macFUSE**, and Linux's official **`libext2fs`** (`e2fsprogs` 1.47.4).

---

## Features

- **Full Read/Write Support**: Open, edit, create, rename, and delete files directly in Finder or terminal.
- **Background Daemon Mode (Default)**: Runs as a background process, automatically releasing your terminal prompt immediately after mount.
- **Automatic Finder Launch**: Automatically opens the mounted volume location in Finder (`open /Volumes/...`) upon successful session launch.
- **Seamless Permission Mapping**: Automatically maps file ownership to your non-root macOS account (`SUDO_USER` / `SUDO_UID` / `SUDO_GID`), eliminating `Permission denied` errors in Finder and terminal.
- **Finder Native Integration**: Browse folders, inspect file sizes, modification dates, and permissions via POSIX macFUSE.
- **Self-Contained Binary**: `libext2fs` C source code is vendored and compiled statically, requiring zero Homebrew dynamic C library dependencies at runtime.
- **Data Integrity & Journal Flushing**: Concurrency safety with single-mutex lock (`Arc<Mutex<Ext2FsHandle>>`) and automatic `ext2fs_flush` on write operations.
- **Built-in Unmount Flag (`-u`)**: Cleanly unmount volumes using `sudo alpaca-extfs -u /Volumes/Ext4Drive`.

---

## Installation

### Via Homebrew

```bash
brew install maxsonferovante/alpaca-extfs-for-mac/alpaca-extfs
```

> **Note**: `alpaca-extfs` requires **macFUSE**. If you haven't installed it yet:
> ```bash
> brew install --cask macfuse
> ```

---

## Prerequisites

1. **macOS 12+**
2. **macFUSE**: Installed on macOS (`brew install --cask macfuse` or download from [macfuse.github.io](https://macfuse.github.io)).
3. **Rust & Cargo** (only if building from source): Standard Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`).

---

## Building

```bash
cargo build --release
```
The compiled self-contained binary will be placed at `./target/release/alpaca-extfs`.

---

## Usage

### Mounting an Ext4 Disk Partition or Image

Mount an Ext4 partition (e.g. `/dev/rdisk4s2`) or a disk image file to a mount point directory. The driver runs as a background daemon by default, opens Finder automatically, and returns your terminal prompt immediately:

```bash
sudo ./target/release/alpaca-extfs /dev/rdisk4s2 /Volumes/Ext4Drive
```

### Options

| Flag | Long Flag | Description |
|---|---|---|
| `-r` | `--read-only` | Mount the volume in Read-Only mode |
| `-u <PATH>` | `--unmount <PATH>` | Safely unmount an active Ext4 volume mount point |
| `-f` | `--foreground` | Run in foreground mode (keeps terminal attached for logs/debugging) |

#### Mount in Read-Only Mode
```bash
sudo ./target/release/alpaca-extfs --read-only /dev/rdisk4s2 /Volumes/Ext4Drive
```

#### Run in Foreground Mode (Debugging)
```bash
sudo ./target/release/alpaca-extfs -f /dev/rdisk4s2 /Volumes/Ext4Drive
```

### Unmounting Safely

To safely unmount the drive and flush all pending Ext4 block/journal changes:

```bash
sudo ./target/release/alpaca-extfs -u /Volumes/Ext4Drive
```

Or using standard macOS unmount:
```bash
sudo umount /Volumes/Ext4Drive
```

---

## Running Integration Tests

The project includes an end-to-end integration test that creates a 16MB loopback Ext4 disk image, performs directory creation, file creation, read/write verification, and validates disk sanity using `e2fsck`:

```bash
cargo test
```

---

## Architecture

```
[ macOS Finder / VFS ]
       │
       ▼
 [ macFUSE Kernel Extension / fuser ]
       │
       ▼
 [ alpaca-extfs (Rust Driver) ]
       │
       ▼
 [ libext2fs (Statically linked e2fsprogs C archive) ]
       │
       ▼
 [ /dev/rdiskXsY (Raw character block device) ]
```

---

## Contributing

Contributions are welcome! If you encounter issues or have feature suggestions:

1. Fork the repository.
2. Create a feature branch (`git checkout -b feature/my-feature`).
3. Ensure all tests pass (`cargo test`).
4. Commit your changes (`git commit -m 'feat: add my feature'`).
5. Push to the branch (`git push origin feature/my-feature`).
6. Open a Pull Request.

---

## License

This project is licensed under the **MIT License**. See the [LICENSE](file:///Users/mferovante/Documents/workspace/alpaca-extfs-for-mac/LICENSE) file for details.

