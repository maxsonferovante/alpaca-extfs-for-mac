# 06 — Integration Test Suite & `e2fsck` Disk Integrity Verification

## What to build

Automated end-to-end integration test suite that generates temporary Ext4 disk images (`dd` + `mkfs.ext4`), mounts them using `mount_ext4`, performs read/write/delete operations, unmounts, and validates the raw disk image using Linux `e2fsck -f` to guarantee 100% filesystem integrity.

User stories covered: #1 through #9 (Complete regression & disk sanity test suite).

## Acceptance criteria

- [ ] Automated integration test script (`tests/integration_test.rs` or bash script).
- [ ] Automate creating Ext4 loopback disk image (`64MB`) using `dd` and `mkfs.ext4`.
- [ ] Mount image via `mount_ext4`, perform file creation, write, readback, and deletion.
- [ ] Unmount image and run `e2fsck -fn /tmp/test_ext4.img` to verify zero corruptions or orphaned blocks.
- [ ] Document test suite instructions in `README.md`.

## Blocked by

- 05 — macFUSE Full Read/Write Finder Integration & Graceful Unmount
