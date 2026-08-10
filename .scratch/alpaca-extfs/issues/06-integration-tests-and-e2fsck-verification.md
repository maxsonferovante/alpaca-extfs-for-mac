# 06 — Integration Test Suite & `e2fsck` Disk Integrity Verification

## What to build

Automated end-to-end integration test suite that generates temporary Ext4 disk images (`dd` + `mkfs.ext4`), mounts them using `mount_ext4`, performs read/write/delete operations, unmounts, and validates the raw disk image using Linux `e2fsck -f` to guarantee 100% filesystem integrity.

User stories covered: #1 through #9 (Complete regression & disk sanity test suite). **Status:** done

## Acceptance criteria

- [x] Automated integration test script (`tests/integration_test.rs`).
- [x] Automate creating Ext4 loopback disk image (`16MB`) using `dd` and `mkfs.ext4`/`mke2fs`.
- [x] Mount image via `Ext2FsHandle`, perform file creation, write, readback, and deletion.
- [x] Unmount image and run `e2fsck -fn /tmp/mount_ext4_integration_test.img` to verify zero corruptions or orphaned blocks.
- [x] Document test suite instructions in `README.md`.

## Blocked by

- 05 — macFUSE Full Read/Write Finder Integration & Graceful Unmount
