use std::ffi::OsStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use libc::{EINVAL, EIO, ENOENT};
use fuser::{
    FileAttr, FileType, Filesystem, KernelConfig, ReplyAttr, ReplyCreate, ReplyData,
    ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyOpen, ReplyStatfs, ReplyWrite, ReplyXattr, Request,
};

use crate::ext2fs::{Ext2FileAttr, Ext2FsHandle};

const TTL: Duration = Duration::from_secs(1);
const FUSE_ROOT_ID: u64 = 1;
const EXT2_ROOT_INO: u32 = 2;

fn fuse_ino_to_ext2(ino: u64) -> u32 {
    if ino == FUSE_ROOT_ID {
        EXT2_ROOT_INO
    } else {
        ino as u32
    }
}

fn ext2_ino_to_fuse(ino: u32) -> u64 {
    if ino == EXT2_ROOT_INO {
        FUSE_ROOT_ID
    } else {
        ino as u64
    }
}

pub struct Ext4FuseFs {
    handle: Arc<Mutex<Ext2FsHandle>>,
    owner_uid: Option<u32>,
    owner_gid: Option<u32>,
}

impl Ext4FuseFs {
    pub fn new(handle: Ext2FsHandle) -> Self {
        Self {
            handle: Arc::new(Mutex::new(handle)),
            owner_uid: None,
            owner_gid: None,
        }
    }

    pub fn with_owner(handle: Ext2FsHandle, uid: u32, gid: u32) -> Self {
        Self {
            handle: Arc::new(Mutex::new(handle)),
            owner_uid: Some(uid),
            owner_gid: Some(gid),
        }
    }
}

fn to_fuser_attr(attr: &Ext2FileAttr, owner_uid: Option<u32>, owner_gid: Option<u32>) -> FileAttr {
    let kind = if attr.is_dir {
        FileType::Directory
    } else if (attr.mode & 0o120000) == 0o120000 {
        FileType::Symlink
    } else {
        FileType::RegularFile
    };

    let perm = if attr.is_dir {
        (attr.mode & 0o777) | 0o755
    } else {
        attr.mode & 0o777
    };

    FileAttr {
        ino: ext2_ino_to_fuse(attr.ino as u32),
        size: attr.size,
        blocks: attr.blocks,
        atime: attr.atime,
        mtime: attr.mtime,
        ctime: attr.ctime,
        crtime: attr.ctime,
        kind,
        perm,
        nlink: attr.nlink,
        uid: owner_uid.unwrap_or(attr.uid),
        gid: owner_gid.unwrap_or(attr.gid),
        rdev: 0,
        blksize: 4096,
        flags: 0,
    }
}

impl Filesystem for Ext4FuseFs {
    fn init(&mut self, _req: &Request<'_>, _config: &mut KernelConfig) -> Result<(), i32> {
        println!("\n==================================================");
        println!("  Ext4 volume mounted successfully and active!");
        println!("  You can now access files in Finder or Terminal.");
        println!("  Press Ctrl+C to unmount cleanly when done.");
        println!("==================================================\n");
        Ok(())
    }

    fn access(&mut self, _req: &Request<'_>, _ino: u64, _mask: i32, reply: ReplyEmpty) {
        reply.ok();
    }

    fn getxattr(&mut self, _req: &Request<'_>, _ino: u64, _name: &OsStr, _size: u32, reply: ReplyXattr) {
        #[cfg(target_os = "macos")]
        reply.error(libc::ENOATTR);
        #[cfg(not(target_os = "macos"))]
        reply.error(libc::ENODATA);
    }

    fn listxattr(&mut self, _req: &Request<'_>, _ino: u64, _size: u32, reply: ReplyXattr) {
        reply.size(0);
    }

    fn statfs(&mut self, _req: &Request<'_>, _ino: u64, reply: ReplyStatfs) {
        let handle = self.handle.lock().unwrap();
        let st = handle.statfs();
        reply.statfs(
            st.blocks,
            st.bfree,
            st.bavail,
            st.files,
            st.ffree,
            st.bsize,
            255,
            st.frsize,
        );
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        let ext2_ino = fuse_ino_to_ext2(ino);
        let handle = self.handle.lock().unwrap();
        match handle.get_attr(ext2_ino) {
            Ok(attr) => reply.attr(&TTL, &to_fuser_attr(&attr, self.owner_uid, self.owner_gid)),
            Err(_) => reply.error(ENOENT),
        }
    }

    fn lookup(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEntry) {
        let name_str = match name.to_str() {
            Some(s) => s,
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        let ext2_parent = fuse_ino_to_ext2(parent);
        let handle = self.handle.lock().unwrap();
        match handle.read_dir(ext2_parent) {
            Ok(entries) => {
                if let Some(entry) = entries.iter().find(|e| e.name == name_str) {
                    match handle.get_attr(entry.inode) {
                        Ok(attr) => reply.entry(&TTL, &to_fuser_attr(&attr, self.owner_uid, self.owner_gid), 0),
                        Err(_) => reply.error(ENOENT),
                    }
                } else {
                    reply.error(ENOENT);
                }
            }
            Err(_) => reply.error(ENOENT),
        }
    }

    fn readdir(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        let ext2_ino = fuse_ino_to_ext2(ino);
        let handle = self.handle.lock().unwrap();
        match handle.read_dir(ext2_ino) {
            Ok(entries) => {
                if offset == 0 {
                    let _ = reply.add(ino, 1, FileType::Directory, ".");
                    let parent_ino = if ino == FUSE_ROOT_ID { FUSE_ROOT_ID } else { 1 };
                    let _ = reply.add(parent_ino, 2, FileType::Directory, "..");
                }

                let start_idx = if offset <= 2 { 0 } else { (offset - 2) as usize };

                for (idx, entry) in entries.iter().enumerate().skip(start_idx) {
                    let file_type = if entry.file_type == 2 {
                        FileType::Directory
                    } else if entry.file_type == 1 {
                        FileType::RegularFile
                    } else if entry.file_type == 7 {
                        FileType::Symlink
                    } else if let Ok(attr) = handle.get_attr(entry.inode) {
                        if attr.is_dir {
                            FileType::Directory
                        } else if (attr.mode & 0o120000) == 0o120000 {
                            FileType::Symlink
                        } else {
                            FileType::RegularFile
                        }
                    } else {
                        FileType::RegularFile
                    };

                    let entry_offset = (idx + 3) as i64;
                    let fuse_entry_ino = ext2_ino_to_fuse(entry.inode);
                    if reply.add(fuse_entry_ino, entry_offset, file_type, &entry.name) {
                        break;
                    }
                }

                reply.ok();
            }
            Err(_) => reply.error(ENOENT),
        }
    }

    fn open(&mut self, _req: &Request<'_>, _ino: u64, _flags: i32, reply: ReplyOpen) {
        reply.opened(0, 0);
    }

    fn read(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyData,
    ) {
        let ext2_ino = fuse_ino_to_ext2(ino);
        let handle = self.handle.lock().unwrap();
        match handle.read_file(ext2_ino, offset as u64, size as usize) {
            Ok(data) => reply.data(&data),
            Err(_) => reply.error(ENOENT),
        }
    }

    fn create(
        &mut self,
        req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        _flags: i32,
        reply: ReplyCreate,
    ) {
        let name_str = match name.to_str() {
            Some(s) => s,
            None => {
                reply.error(EINVAL);
                return;
            }
        };

        let uid = self.owner_uid.unwrap_or_else(|| req.uid());
        let gid = self.owner_gid.unwrap_or_else(|| req.gid());
        let ext2_parent = fuse_ino_to_ext2(parent);
        let handle = self.handle.lock().unwrap();
        match handle.create_file(ext2_parent, name_str, mode as u16, uid, gid) {
            Ok(ino) => match handle.get_attr(ino) {
                Ok(attr) => reply.created(&TTL, &to_fuser_attr(&attr, self.owner_uid, self.owner_gid), 0, 0, 0),
                Err(_) => reply.error(EIO),
            },
            Err(_) => reply.error(EIO),
        }
    }

    fn write(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: ReplyWrite,
    ) {
        let ext2_ino = fuse_ino_to_ext2(ino);
        let handle = self.handle.lock().unwrap();
        match handle.write_file(ext2_ino, offset as u64, data) {
            Ok(written) => reply.written(written as u32),
            Err(_) => reply.error(EIO),
        }
    }

    fn unlink(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let name_str = match name.to_str() {
            Some(s) => s,
            None => {
                reply.error(EINVAL);
                return;
            }
        };

        let ext2_parent = fuse_ino_to_ext2(parent);
        let handle = self.handle.lock().unwrap();
        match handle.unlink(ext2_parent, name_str) {
            Ok(_) => reply.ok(),
            Err(_) => reply.error(EIO),
        }
    }

    fn mkdir(
        &mut self,
        req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        let name_str = match name.to_str() {
            Some(s) => s,
            None => {
                reply.error(EINVAL);
                return;
            }
        };

        let uid = self.owner_uid.unwrap_or_else(|| req.uid());
        let gid = self.owner_gid.unwrap_or_else(|| req.gid());
        let ext2_parent = fuse_ino_to_ext2(parent);
        let handle = self.handle.lock().unwrap();
        match handle.mkdir(ext2_parent, name_str, mode as u16, uid, gid) {
            Ok(ino) => match handle.get_attr(ino) {
                Ok(attr) => reply.entry(&TTL, &to_fuser_attr(&attr, self.owner_uid, self.owner_gid), 0),
                Err(_) => reply.error(EIO),
            },
            Err(_) => reply.error(EIO),
        }
    }

    fn rmdir(&mut self, _req: &Request<'_>, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let name_str = match name.to_str() {
            Some(s) => s,
            None => {
                reply.error(EINVAL);
                return;
            }
        };

        let ext2_parent = fuse_ino_to_ext2(parent);
        let handle = self.handle.lock().unwrap();
        match handle.unlink(ext2_parent, name_str) {
            Ok(_) => reply.ok(),
            Err(_) => reply.error(EIO),
        }
    }

    fn flush(&mut self, _req: &Request<'_>, _ino: u64, _fh: u64, _lock_owner: u64, reply: ReplyEmpty) {
        let handle = self.handle.lock().unwrap();
        let _ = handle.flush();
        reply.ok();
    }
}

#[cfg(target_os = "macos")]
pub mod macos_fuse {
    use std::ffi::CString;
    use std::io;
    use std::os::fd::{FromRawFd, OwnedFd};
    use std::path::Path;
    use fuser::{Filesystem, MountOption, Session, SessionACL};

    #[repr(C)]
    struct FuseArgs {
        argc: i32,
        argv: *const *const i8,
        allocated: i32,
    }

    #[link(name = "fuse")]
    extern "C" {
        fn fuse_mount(dir: *const i8, args: *const FuseArgs) -> *mut std::ffi::c_void;
        fn fuse_chan_fd(chan: *mut std::ffi::c_void) -> i32;
        fn fuse_unmount(dir: *const i8, chan: *mut std::ffi::c_void);
    }

    pub struct MacFuseSession {
        target_cstring: CString,
        chan: *mut std::ffi::c_void,
    }

    impl Drop for MacFuseSession {
        fn drop(&mut self) {
            if !self.chan.is_null() {
                unsafe {
                    fuse_unmount(self.target_cstring.as_ptr(), self.chan);
                }
            }
        }
    }

    pub fn mount<FS: Filesystem, P: AsRef<Path>>(
        filesystem: FS,
        mountpoint: P,
        options: &[MountOption],
    ) -> io::Result<()> {
        let mountpoint = mountpoint.as_ref();
        let target = CString::new(
            mountpoint
                .to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid mountpoint path"))?,
        )?;

        let mut args_vec = vec![CString::new("alpaca-extfs").unwrap()];
        args_vec.push(CString::new("-o").unwrap());
        args_vec.push(CString::new("allow_recursion").unwrap());
        args_vec.push(CString::new("-o").unwrap());
        args_vec.push(CString::new("local").unwrap());
        args_vec.push(CString::new("-o").unwrap());
        args_vec.push(CString::new("defer_permissions").unwrap());

        for opt in options {
            let s = match opt {
                MountOption::FSName(name) => format!("fsname={}", name),
                MountOption::Subtype(subtype) => format!("subtype={}", subtype),
                MountOption::CUSTOM(val) => val.clone(),
                MountOption::RO => "ro".to_string(),
                MountOption::RW => "rw".to_string(),
                MountOption::AllowOther => "allow_other".to_string(),
                MountOption::AllowRoot => "allow_root".to_string(),
                MountOption::DefaultPermissions => "default_permissions".to_string(),
                _ => continue,
            };
            args_vec.push(CString::new("-o").unwrap());
            args_vec.push(CString::new(s).unwrap());
        }

        let argptrs: Vec<*const i8> = args_vec.iter().map(|s| s.as_ptr()).collect();
        let fuse_args = FuseArgs {
            argc: argptrs.len() as i32,
            argv: argptrs.as_ptr(),
            allocated: 0,
        };

        let chan = unsafe { fuse_mount(target.as_ptr(), &fuse_args) };
        if chan.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "macFUSE fuse_mount failed for '{}'. If already mounted, unmount first using: 'sudo umount {}'",
                    mountpoint.display(),
                    mountpoint.display()
                ),
            ));
        }

        let raw_fd = unsafe { fuse_chan_fd(chan) };
        if raw_fd < 0 {
            unsafe {
                fuse_unmount(target.as_ptr(), chan);
            }
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "macFUSE returned invalid file descriptor for '{}'. If already mounted, unmount first using: 'sudo umount {}'",
                    mountpoint.display(),
                    mountpoint.display()
                ),
            ));
        }

        let dup_fd = unsafe { libc::dup(raw_fd) };
        if dup_fd < 0 {
            unsafe {
                fuse_unmount(target.as_ptr(), chan);
            }
            return Err(io::Error::last_os_error());
        }

        let owned_fd = unsafe { OwnedFd::from_raw_fd(dup_fd) };
        let acl = if options.contains(&MountOption::AllowOther) {
            SessionACL::All
        } else if options.contains(&MountOption::AllowRoot) {
            SessionACL::RootAndOwner
        } else {
            SessionACL::Owner
        };

        let _guard = MacFuseSession {
            target_cstring: target,
            chan,
        };

        let mut session = Session::from_fd(filesystem, owned_fd, acl);
        session.run()
    }
}

pub fn mount_filesystem<FS: fuser::Filesystem, P: AsRef<std::path::Path>>(
    filesystem: FS,
    mountpoint: P,
    options: &[fuser::MountOption],
) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos_fuse::mount(filesystem, mountpoint, options)
    }
    #[cfg(not(target_os = "macos"))]
    {
        fuser::mount2(filesystem, mountpoint, options)
    }
}

