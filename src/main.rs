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
    options.push(MountOption::FSName(args.device.to_string_lossy().to_string()));

    // Carrega a extensão do macFUSE no kernel se estiver presente
    let load_macfuse_bin = "/Library/Filesystems/macfuse.fs/Contents/Resources/load_macfuse";
    if std::path::Path::new(load_macfuse_bin).exists() {
        let _ = std::process::Command::new(load_macfuse_bin).status();
    }

    if !args.mount_point.exists() {
        if let Err(e) = std::fs::create_dir_all(&args.mount_point) {
            eprintln!("Failed to create mount point directory '{}': {}", args.mount_point.display(), e);
            std::process::exit(1);
        }
    }

    println!("Initializing macFUSE session on '{}'...", args.mount_point.display());

    if let Err(e) = fuse::mount_filesystem(fs, &args.mount_point, &options) {
        eprintln!("\nError mounting macFUSE volume: {}", e);
        eprintln!("\n--- DIAGNOSTIC HINT ---");
        eprintln!("On macOS, macFUSE requires kernel/system extension permission.");
        eprintln!("1. Open System Settings -> Privacy & Security.");
        eprintln!("2. Scroll down to 'Security' section.");
        eprintln!("3. Click 'Allow' to authorize the macFUSE extension if prompted.");
        eprintln!("4. Alternatively, try mounting to an empty directory in /tmp (e.g. /tmp/ext4_mount).");
        std::process::exit(1);
    }

    println!("Ext4 volume unmounted cleanly.");
}

