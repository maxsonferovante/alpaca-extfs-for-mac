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
    #[arg(required_unless_present = "unmount")]
    device: Option<PathBuf>,

    /// Target mount point directory (e.g., /Volumes/Ext4Drive or /tmp/mount)
    #[arg(required_unless_present = "unmount")]
    mount_point: Option<PathBuf>,

    /// Read-only mode
    #[arg(short, long, default_value_t = false)]
    read_only: bool,

    /// Unmount an active Ext4 volume mount point
    #[arg(short, long)]
    unmount: Option<PathBuf>,

    /// Run in foreground mode instead of daemonizing into background
    #[arg(short, long, default_value_t = false)]
    foreground: bool,

    /// Internal child worker flag
    #[arg(long, default_value_t = false, hide = true)]
    child: bool,
}

fn main() {
    let args = Cli::parse();

    if let Some(target) = args.unmount {
        println!("Unmounting Ext4 volume at '{}'...", target.display());
        let status = std::process::Command::new("umount")
            .arg(&target)
            .status();
        match status {
            Ok(s) if s.success() => {
                println!("Successfully unmounted '{}'.", target.display());
                std::process::exit(0);
            }
            _ => {
                let status2 = std::process::Command::new("diskutil")
                    .args(&["unmount", "force", target.to_str().unwrap()])
                    .status();
                if let Ok(s) = status2 {
                    if s.success() {
                        println!("Successfully unmounted '{}'.", target.display());
                        std::process::exit(0);
                    }
                }
                let _ = std::process::Command::new("umount")
                    .args(&["-f", target.to_str().unwrap()])
                    .status();
                println!("Unmount command sent for '{}'.", target.display());
                std::process::exit(0);
            }
        }
    }

    let device = args.device.expect("Device path required");
    let mount_point = args.mount_point.expect("Mount point path required");

    let sudo_uid: Option<u32> = std::env::var("SUDO_UID").ok().and_then(|v| v.parse().ok());
    let sudo_gid: Option<u32> = std::env::var("SUDO_GID").ok().and_then(|v| v.parse().ok());

    if !args.foreground && !args.child {
        println!(
            "Mounting Ext4 volume '{}' to '{}' in background...",
            device.display(),
            mount_point.display()
        );

        let current_exe = std::env::current_exe().expect("Failed to locate current executable");
        let mut cmd = std::process::Command::new(current_exe);
        cmd.arg(&device)
            .arg(&mount_point)
            .arg("--child");
        if args.read_only {
            cmd.arg("--read-only");
        }

        let mut child_proc = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to spawn background daemon: {}", e);
                std::process::exit(1);
            }
        };

        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if let Ok(Some(status)) = child_proc.try_wait() {
                if !status.success() {
                    eprintln!("Background daemon failed to start FUSE session.");
                    std::process::exit(1);
                }
            }
            if mount_point.exists() {
                break;
            }
        }

        println!("\n✅ Ext4 volume mounted successfully at '{}'!", mount_point.display());
        println!("Opening Finder...");
        let sudo_user = std::env::var("SUDO_USER").unwrap_or_default();
        let open_status = if !sudo_user.is_empty() {
            std::process::Command::new("sudo")
                .args(&["-u", &sudo_user, "open", mount_point.to_str().unwrap()])
                .status()
        } else {
            std::process::Command::new("open").arg(&mount_point).status()
        };

        if open_status.is_err() || !open_status.as_ref().map_or(false, |s| s.success()) {
            let _ = std::process::Command::new("open").arg(&mount_point).status();
        }

        println!("To unmount, run: sudo alpaca-extfs -u {}", mount_point.display());
        std::process::exit(0);
    }

    let handle = match ext2fs::Ext2FsHandle::open(&device, args.read_only) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Error opening Ext4 filesystem: {}", e);
            std::process::exit(1);
        }
    };

    let fs = if let (Some(uid), Some(gid)) = (sudo_uid, sudo_gid) {
        fuse::Ext4FuseFs::with_owner(handle, uid, gid)
    } else {
        fuse::Ext4FuseFs::new(handle)
    };

    let volname = mount_point
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Ext4Drive");

    let mut options = Vec::new();
    if args.read_only {
        options.push(MountOption::RO);
    } else {
        options.push(MountOption::RW);
    }

    options.push(MountOption::AllowOther);
    options.push(MountOption::CUSTOM(format!("volname={}", volname)));
    options.push(MountOption::FSName(device.to_string_lossy().to_string()));

    let load_macfuse_bin = "/Library/Filesystems/macfuse.fs/Contents/Resources/load_macfuse";
    if std::path::Path::new(load_macfuse_bin).exists() {
        let _ = std::process::Command::new(load_macfuse_bin).status();
    }

    // Enable allow_other in kernel sysctl for macFUSE
    let _ = std::process::Command::new("sysctl")
        .arg("-w")
        .arg("vfs.generic.macfuse.tunables.allow_other=1")
        .status();

    if !mount_point.exists() {
        if let Err(e) = std::fs::create_dir_all(&mount_point) {
            eprintln!("Failed to create mount point directory '{}': {}", mount_point.display(), e);
            std::process::exit(1);
        }
    }

    if args.foreground {
        println!("Initializing macFUSE session on '{}'...", mount_point.display());
        let sudo_user = std::env::var("SUDO_USER").unwrap_or_default();
        if !sudo_user.is_empty() {
            let _ = std::process::Command::new("sudo")
                .args(&["-u", &sudo_user, "open", mount_point.to_str().unwrap()])
                .status();
        } else {
            let _ = std::process::Command::new("open").arg(&mount_point).status();
        }
    }

    if let Err(e) = fuse::mount_filesystem(fs, &mount_point, &options) {
        eprintln!("\nError mounting macFUSE volume: {}", e);
        eprintln!("\n--- DIAGNOSTIC HINT ---");
        eprintln!("On macOS, macFUSE requires kernel/system extension permission.");
        eprintln!("1. Open System Settings -> Privacy & Security.");
        eprintln!("2. Scroll down to 'Security' section.");
        eprintln!("3. Click 'Allow' to authorize the macFUSE extension if prompted.");
        eprintln!("4. Alternatively, try mounting to an empty directory in /tmp (e.g. /tmp/ext4_mount).");
        std::process::exit(1);
    }

    if args.foreground {
        println!("Ext4 volume unmounted cleanly.");
    }
}

