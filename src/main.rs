use clap::Parser;
use fuser::MountOption;
use std::path::PathBuf;

pub mod ext2fs;
pub mod ffi;
pub mod fuse;

#[derive(Parser, Debug)]
#[command(
    name = "alpaca-extfs",
    version,
    about = "Ext4 Read/Write driver for macOS integrated with Finder"
)]
struct Cli {

    /// Device path (e.g., /dev/rdisk4s2 or disk.img)
    #[arg(required = true)]
    device: PathBuf,

    /// Target mount point directory (e.g., /Volumes/Ext4Drive or /tmp/mount)
    #[arg(required = true)]
    mount_point: PathBuf,

    /// Read-only mode
    #[arg(short, long, default_value_t = false)]
    read_only: bool,
}

fn main() {
    let args = Cli::parse();

    println!(
        "Mounting Ext4 volume '{}' to '{}'...",
        args.device.display(),
        args.mount_point.display()
    );

    let handle = match ext2fs::Ext2FsHandle::open(&args.device, args.read_only) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error opening Ext4 filesystem: {}", e);
            std::process::exit(1);
        }
    };

    let fs = fuse::Ext4FuseFs::new(handle);

    let volname = args
        .mount_point
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Ext4Drive");

    let mut options = Vec::new();
    if args.read_only {
        options.push(MountOption::RO);
    } else {
        options.push(MountOption::RW);
    }

    options.push(MountOption::CUSTOM(format!("volname={}", volname)));



    println!("Ext4 volume mounted successfully. Press Ctrl+C to unmount.");
    if let Err(e) = fuser::mount2(fs, &args.mount_point, &options) {
        eprintln!("Error mounting macFUSE volume: {}", e);
        std::process::exit(1);
    }
}
