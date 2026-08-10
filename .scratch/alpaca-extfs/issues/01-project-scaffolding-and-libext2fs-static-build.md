# 01 — Project Scaffolding & Vendored `libext2fs` Static Compilation

## What to build

Cargo Rust project setup with vendored `libext2fs` C source code (`e2fsprogs`) compiled statically via Cargo `build.rs` and `cc` crate. Building the project produces a self-contained Rust binary with zero dynamic Homebrew runtime library linkage errors.

User story covered: #8 (Self-contained binary build without runtime Homebrew dependencies).

**Status:** done

## Acceptance criteria

- [x] Cargo binary project initialized with `fuser`, `libc`, `nix`, `cc`, and `bindgen` dependencies in `Cargo.toml`.
- [x] Vendored `e2fsprogs` `libext2fs` C source present under `vendor/e2fsprogs`.
- [x] Implement `build.rs` script to compile `libext2fs` as a static C archive (`libext2fs.a`).
- [x] `cargo build` completes successfully and produces an executable binary without `.dylib` linking errors.

## Blocked by

None — can start immediately.
