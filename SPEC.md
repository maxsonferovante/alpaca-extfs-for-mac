# Technical Specification: Background Daemon Mode, Auto Finder Launch & Permission Mapping

## Problem Statement

When mounting an Ext4 volume using `alpaca-extfs`, the CLI command blocks the terminal in foreground indefinitely without releasing the command prompt. Furthermore, when mounted via `sudo`, macFUSE restricts filesystem access exclusively to root (UID 0), causing non-root macOS users (`mferovante`) and Finder to get `Permission denied` errors when trying to view or access files. Finally, Finder does not automatically open the mounted volume location upon mounting.

## Solution

Automate volume mounting and Finder interaction by running the FUSE background daemon process by default, mapping file ownership attributes to `SUDO_USER` with `allow_other` enabled, launching Finder automatically (`open /Volumes/Ext4Drive`) upon successful mount, releasing the terminal prompt immediately, and providing an explicit `--unmount` CLI flag.

## User Stories

1. As a macOS user, I want `alpaca-extfs` to run as a background daemon by default, so that my terminal prompt is released immediately after mounting.
2. As a macOS user, I want Finder to open automatically to the mount point upon successful mounting, so that I can immediately interact with my files without manual navigation.
3. As a macOS user, I want mounted Ext4 files to be accessible by my non-root user account when executed via `sudo`, so that I do not encounter `Permission denied` errors in Finder or terminal.
4. As a macOS user, I want `alpaca-extfs` to map file ownership UID/GID to `SUDO_USER` when executed with `sudo`, so that Finder presents all files as owned by my active account.
5. As a macOS user, I want to unmount the volume using `alpaca-extfs --unmount /Volumes/Ext4Drive`, so that I have a convenient CLI alternative to `sudo umount` or Finder eject.
6. As a developer, I want a `--foreground` (`-f`) flag, so that I can run the driver in the foreground for debugging and log inspection when needed.
7. As a macOS user, I want clear status messages confirming when the background daemon starts and when Finder opens, so that I know the filesystem is ready.
8. As a macOS user, I want invalid/duplicate mount attempts to report clear diagnostic messages advising how to unmount stale mounts, so that I can resolve conflicts quickly.

## Implementation Decisions

- **Daemonization & Process Lifecycle**:
  - Default execution mode detaches the main process into a background process (or spawns a background worker handling `Session::run`).
  - The parent CLI process waits for confirmation of FUSE session initialization, executes `open <mountpoint>`, prints success confirmation, and exits cleanly back to the terminal prompt.
  - Added `-f` / `--foreground` flag to keep execution in foreground when requested.
  - Added `-u` / `--unmount <mountpoint>` flag to invoke clean unmounting.

- **FUSE Mount Options & Permission Mapping**:
  - `allow_other` is enabled for macFUSE session options.
  - `allow_recursion` is enabled to allow mounting under paths residing within macFUSE volume structures.
  - `SUDO_USER`, `SUDO_UID`, and `SUDO_GID` are inspected at runtime from environment variables.
  - File attributes (`to_fuser_attr`) dynamically return `uid` and `gid` matching `SUDO_UID` / `SUDO_GID` when executed under `sudo`, ensuring full read/write access for the logged-in desktop user.

- **macOS Finder Integration**:
  - Upon successful background session initialization, `Command::new("open").arg(mountpoint).status()` is invoked.

## Testing Decisions

- **Good Test Criteria**: Tests should verify external end-to-end behavior (drive mounting, daemon process status, permission mapping, unmounting, and filesystem integrity) without depending on internal implementation details.
- **Modules Tested**:
  - `tests/integration_test.rs`: End-to-end integration test covering loopback image creation, background daemon execution, permission verification, unmounting, and `e2fsck` validation.
  - `src/fuse.rs`: Unit test for attribute translation and `statfs` calculations.
- **Prior Art**: `tests/integration_test.rs` already contains loopback image formatting with `mke2fs` and validation with `e2fsck`.

## Out of Scope

- Graphical macOS menu bar application or GUI tray app.
- Automatic disk insertion auto-detection (DiskArbitration daemon).
- Multi-user ACL permission translation beyond `SUDO_USER` UID/GID mapping.

## Further Notes

- Existing unit and integration tests remain passing (`3 passed`).
- Compatibility with macFUSE 4.x / 5.x on macOS Sonoma and Sequoia is maintained using the `fuse_mount` / `fuse_chan_fd` native channel binding.
