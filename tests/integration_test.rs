use std::process::Command;
use std::fs;
use std::path::Path;

use alpaca_extfs::ext2fs::Ext2FsHandle;


#[test]
fn test_ext4_read_write_directory_and_e2fsck_integrity() {
    let img_path = "/tmp/mount_ext4_integration_test.img";

    if Path::new(img_path).exists() {
        let _ = fs::remove_file(img_path);
    }

    // 1. Create a 16MB empty file
    let status = Command::new("dd")
        .args(&["if=/dev/zero", &format!("of={}", img_path), "bs=1M", "count=16"])
        .status()
        .expect("Failed to execute dd");
    assert!(status.success(), "dd command failed");

    // 2. Format as ext4 using mke2fs
    let mke2fs_bin = if Path::new("/opt/homebrew/opt/e2fsprogs/sbin/mke2fs").exists() {
        "/opt/homebrew/opt/e2fsprogs/sbin/mke2fs"
    } else {
        "mke2fs"
    };

    let status = Command::new(mke2fs_bin)
        .args(&["-t", "ext4", "-F", img_path])
        .status()
        .expect("Failed to execute mke2fs");
    assert!(status.success(), "mke2fs formatting failed");

    // 3. Open Ext4 filesystem with Ext2FsHandle
    let handle = Ext2FsHandle::open(img_path, false)
        .expect("Ext2FsHandle failed to open newly created ext4 image");

    // 4. Create directory 'docs'
    let root_ino = Ext2FsHandle::root_ino();
    let _dir_ino = handle

        .mkdir(root_ino, "docs", 0o755, 501, 20)
        .expect("Failed to create 'docs' directory");

    // 5. Create file 'hello.txt' inside root
    let file_ino = handle
        .create_file(root_ino, "hello.txt", 0o644, 501, 20)
        .expect("Failed to create 'hello.txt'");

    // 6. Write content to 'hello.txt'
    let content = b"Hello Ext4 from macOS Rust driver!";
    let written = handle
        .write_file(file_ino, 0, content)
        .expect("Failed to write to 'hello.txt'");
    assert_eq!(written, content.len());

    // 7. Read back content from 'hello.txt'
    let read_back = handle
        .read_file(file_ino, 0, content.len())
        .expect("Failed to read from 'hello.txt'");
    assert_eq!(read_back, content);

    // 8. List directory entries
    let entries = handle
        .read_dir(root_ino)
        .expect("Failed to read root directory entries");
    let names: Vec<String> = entries.into_iter().map(|e| e.name).collect();
    assert!(names.contains(&"docs".to_string()));
    assert!(names.contains(&"hello.txt".to_string()));

    // 9. Flush and close handle
    handle.flush().expect("Failed to flush filesystem");
    drop(handle);

    // 10. Run e2fsck -fn to verify 100% filesystem integrity
    let e2fsck_bin = if Path::new("/opt/homebrew/opt/e2fsprogs/sbin/e2fsck").exists() {
        "/opt/homebrew/opt/e2fsprogs/sbin/e2fsck"
    } else {
        "e2fsck"
    };

    let status = Command::new(e2fsck_bin)
        .args(&["-fn", img_path])
        .status()
        .expect("Failed to execute e2fsck");
    assert!(status.success(), "e2fsck reported filesystem errors or corruption!");

    // Clean up
    let _ = fs::remove_file(img_path);
}
