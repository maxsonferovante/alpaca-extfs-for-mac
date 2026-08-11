# Project Implementation Issues

## Issue 1: Permission Mapping & allow_other FUSE Session Setup

### What to build

Enable `allow_other` in macFUSE mount options, detect `SUDO_USER`, `SUDO_UID`, and `SUDO_GID` from environment variables, and map file ownership attributes in FUSE responses (`to_fuser_attr`, `create`, `mkdir`) to `SUDO_UID`/`SUDO_GID` so non-root macOS users and Finder have full read/write access without `Permission denied` errors.

### Acceptance criteria

- [ ] `MountOption::AllowOther` is included in macFUSE mount parameters.
- [ ] `SUDO_USER`, `SUDO_UID`, and `SUDO_GID` are detected when executed via `sudo`.
- [ ] `Ext4FuseFs` returns `uid` and `gid` matching `SUDO_UID`/`SUDO_GID` for all inode attribute queries.
- [ ] Running `ls -la /Volumes/Ext4Drive` as the non-root user succeeds without `Permission denied`.
- [ ] Automated tests pass.

### Blocked by

None - can start immediately

---

## Issue 2: Background Daemon Process Execution & Auto Finder Launch

### What to build

Implement default background daemon mode for `alpaca-extfs`. When executed without `-f`, the process spawns a background worker handling the FUSE session loop, waits for session initialization, executes `open <mountpoint>` to launch Finder, prints confirmation output, and releases the terminal prompt immediately. Include a `-f` / `--foreground` CLI flag for debugging.

### Acceptance criteria

- [ ] Command prompt is released immediately after background session initialization.
- [ ] Finder opens the mount point directory (`open /Volumes/Ext4Drive`) automatically.
- [ ] Passing `-f` or `--foreground` keeps execution in foreground mode.
- [ ] Terminal prints confirmation banner with mount point path upon successful background launch.

### Blocked by

- Issue 1

---

## Issue 3: CLI Unmount Flag & Active Mount Diagnostics

### What to build

Add a `-u` / `--unmount <mountpoint>` CLI flag to `alpaca-extfs` that safely unmounts active volumes (`sudo umount <mountpoint>`). Update error diagnostics when mounting over an active or stale macFUSE mount point to report clear messages recommending unmount commands.

### Acceptance criteria

- [ ] `sudo alpaca-extfs -u /Volumes/Ext4Drive` unmounts the volume cleanly and reports status.
- [ ] Attempting to mount over an active mount point reports diagnostic error advising how to unmount first.
- [ ] Integration tests verify mount and unmount workflows.

### Blocked by

- Issue 1
- Issue 2
