use std::ffi::OsStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use libc::{EINVAL, EIO, ENOENT};
use fuser::{
    FileAttr, FileType, Filesystem, KernelConfig, ReplyAttr, ReplyCreate, ReplyData,
    ReplyDirectory, ReplyEmpty, ReplyEntry, ReplyOpen, ReplyWrite, Request,
};

use crate::ext2fs::{Ext2FileAttr, Ext2FsHandle};

const TTL: Duration = Duration::from_secs(1);

pub struct Ext4FuseFs {
    handle: Arc<Mutex<Ext2FsHandle>>,
}

impl Ext4FuseFs {
    pub fn new(handle: Ext2FsHandle) -> Self {
        Self {
            handle: Arc::new(Mutex::new(handle)),
        }
    }
}

fn to_fuser_attr(attr: &Ext2FileAttr) -> FileAttr {
    let kind = if attr.is_dir {
        FileType::Directory
    } else if (attr.mode & 0o120000) == 0o120000 {
        FileType::Symlink
    } else {
        FileType::RegularFile
    };

    FileAttr {
        ino: attr.ino,
        size: attr.size,
        blocks: attr.blocks,
        atime: attr.atime,
        mtime: attr.mtime,
        ctime: attr.ctime,
        crtime: attr.ctime,
        kind,
        perm: (attr.mode & 0o777),
        nlink: attr.nlink,
        uid: attr.uid,
        gid: attr.gid,
        rdev: 0,
        blksize: 4096,
        flags: 0,
    }
}

impl Filesystem for Ext4FuseFs {
    fn init(&mut self, _req: &Request<'_>, _config: &mut KernelConfig) -> Result<(), i32> {
        Ok(())
    }

    fn getattr(&mut self, _req: &Request<'_>, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        let handle = self.handle.lock().unwrap();
        match handle.get_attr(ino as u32) {
            Ok(attr) => reply.attr(&TTL, &to_fuser_attr(&attr)),
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

        let handle = self.handle.lock().unwrap();
        match handle.read_dir(parent as u32) {
            Ok(entries) => {
                if let Some(entry) = entries.iter().find(|e| e.name == name_str) {
                    match handle.get_attr(entry.inode) {
                        Ok(attr) => reply.entry(&TTL, &to_fuser_attr(&attr), 0),
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
        let handle = self.handle.lock().unwrap();
        match handle.read_dir(ino as u32) {
            Ok(entries) => {
                if offset == 0 {
                    let _ = reply.add(ino, 1, FileType::Directory, ".");
                    let _ = reply.add(1, 2, FileType::Directory, "..");
                }

                let start_idx = if offset <= 2 { 0 } else { (offset - 2) as usize };

                for (idx, entry) in entries.iter().enumerate().skip(start_idx) {
                    let file_type = if entry.file_type == 2 {
                        FileType::Directory
                    } else {
                        FileType::RegularFile
                    };

                    let entry_offset = (idx + 3) as i64;
                    if reply.add(entry.inode as u64, entry_offset, file_type, &entry.name) {
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
        let handle = self.handle.lock().unwrap();
        match handle.read_file(ino as u32, offset as u64, size as usize) {
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

        let handle = self.handle.lock().unwrap();
        match handle.create_file(parent as u32, name_str, mode as u16, req.uid(), req.gid()) {
            Ok(ino) => match handle.get_attr(ino) {
                Ok(attr) => reply.created(&TTL, &to_fuser_attr(&attr), 0, 0, 0),
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
        let handle = self.handle.lock().unwrap();
        match handle.write_file(ino as u32, offset as u64, data) {
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

        let handle = self.handle.lock().unwrap();
        match handle.unlink(parent as u32, name_str) {
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

        let handle = self.handle.lock().unwrap();
        match handle.mkdir(parent as u32, name_str, mode as u16, req.uid(), req.gid()) {
            Ok(ino) => match handle.get_attr(ino) {
                Ok(attr) => reply.entry(&TTL, &to_fuser_attr(&attr), 0),
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

        let handle = self.handle.lock().unwrap();
        match handle.unlink(parent as u32, name_str) {
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
