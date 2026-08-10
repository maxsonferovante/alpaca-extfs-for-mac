use std::ffi::CString;

use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use std::time::{Duration, UNIX_EPOCH, SystemTime};

use crate::ffi::bindings::*;

pub struct Ext2FsHandle {
    fs: ext2_filsys,
}

unsafe impl Send for Ext2FsHandle {}
unsafe impl Sync for Ext2FsHandle {}

#[derive(Debug, Clone)]
pub struct Ext2DirEntry {
    pub inode: u32,
    pub name: String,
    pub file_type: u8,
}

#[derive(Debug, Clone)]
pub struct Ext2FileAttr {
    pub ino: u64,
    pub size: u64,
    pub blocks: u64,
    pub atime: SystemTime,
    pub mtime: SystemTime,
    pub ctime: SystemTime,
    pub mode: u16,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub is_dir: bool,
}

impl Ext2FsHandle {
    pub fn open<P: AsRef<Path>>(path: P, read_only: bool) -> io::Result<Self> {
        let c_path = CString::new(path.as_ref().as_os_str().as_bytes())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        let flags = if read_only {
            0
        } else {
            EXT2_FLAG_RW
        } | EXT2_FLAG_FORCE;

        let mut fs: ext2_filsys = ptr::null_mut();
        let retval = unsafe {
            ext2fs_open(
                c_path.as_ptr(),
                flags as i32,
                0,
                0,
                unix_io_manager,
                &mut fs,
            )
        };


        if retval != 0 || fs.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to open ext4 filesystem (error code: {})", retval),
            ));
        }

        let handle = Self { fs };
        if !read_only {
            let _ = handle.read_bitmaps();
        }

        Ok(handle)
    }

    pub fn read_bitmaps(&self) -> io::Result<()> {
        let retval = unsafe { ext2fs_read_bitmaps(self.fs) };
        if retval != 0 {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read ext4 bitmaps: {}", retval),
            ))
        } else {
            Ok(())
        }
    }

    pub fn root_ino() -> u32 {
        EXT2_ROOT_INO
    }

    pub fn namei(&self, path: &str) -> io::Result<u32> {
        if path == "/" || path.is_empty() {
            return Ok(EXT2_ROOT_INO);
        }

        let c_path = CString::new(path)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

        let mut res_ino: ext2_ino_t = 0;
        let retval = unsafe {
            ext2fs_namei(
                self.fs,
                EXT2_ROOT_INO,
                EXT2_ROOT_INO,
                c_path.as_ptr(),
                &mut res_ino,
            )
        };

        if retval != 0 {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Path '{}' not found in ext4 filesystem", path),
            ))
        } else {
            Ok(res_ino)
        }
    }

    pub fn read_inode(&self, ino: u32) -> io::Result<ext2_inode> {
        let mut inode: ext2_inode = unsafe { std::mem::zeroed() };
        let retval = unsafe { ext2fs_read_inode(self.fs, ino, &mut inode) };
        if retval != 0 {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read inode {}: {}", ino, retval),
            ))
        } else {
            Ok(inode)
        }
    }

    pub fn get_attr(&self, ino: u32) -> io::Result<Ext2FileAttr> {
        let inode = self.read_inode(ino)?;

        let size = (inode.i_size as u64) | ((inode.i_size_high as u64) << 32);
        let mode = inode.i_mode;
        let is_dir = (mode & 0o170000) == 0o040000;

        let atime = UNIX_EPOCH + Duration::from_secs(inode.i_atime as u64);
        let mtime = UNIX_EPOCH + Duration::from_secs(inode.i_mtime as u64);
        let ctime = UNIX_EPOCH + Duration::from_secs(inode.i_ctime as u64);

        Ok(Ext2FileAttr {
            ino: ino as u64,
            size,
            blocks: inode.i_blocks as u64,
            atime,
            mtime,
            ctime,
            mode,
            nlink: inode.i_links_count as u32,

            uid: inode.i_uid as u32,
            gid: inode.i_gid as u32,
            is_dir,
        })
    }

    pub fn read_dir(&self, dir_ino: u32) -> io::Result<Vec<Ext2DirEntry>> {
        struct DirContext {
            entries: Vec<Ext2DirEntry>,
        }

        unsafe extern "C" fn dir_proc(
            _dir: ext2_ino_t,
            _entry: std::os::raw::c_int,
            dirent: *mut ext2_dir_entry,
            _offset: std::os::raw::c_int,
            _blocksize: std::os::raw::c_int,
            _buf: *mut std::os::raw::c_char,
            priv_data: *mut std::os::raw::c_void,
        ) -> std::os::raw::c_int {
            if dirent.is_null() || priv_data.is_null() {
                return 0;
            }

            let ctx = &mut *(priv_data as *mut DirContext);
            let entry = &*dirent;

            let name_len = (entry.name_len & 0xFF) as usize;
            if name_len > 0 {
                let name_bytes = std::slice::from_raw_parts(entry.name.as_ptr() as *const u8, name_len);
                if let Ok(name) = std::str::from_utf8(name_bytes) {
                    if name != "." && name != ".." {
                        let file_type = (entry.name_len >> 8) as u8;
                        ctx.entries.push(Ext2DirEntry {
                            inode: entry.inode,
                            name: name.to_string(),
                            file_type,
                        });
                    }
                }
            }
            0
        }

        let mut ctx = DirContext {
            entries: Vec::new(),
        };

        let retval = unsafe {
            ext2fs_dir_iterate2(
                self.fs,
                dir_ino,
                0,
                ptr::null_mut(),
                Some(dir_proc),
                &mut ctx as *mut _ as *mut _,
            )
        };

        if retval != 0 {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to iterate directory inode {}: {}", dir_ino, retval),
            ))
        } else {
            Ok(ctx.entries)
        }
    }

    pub fn read_file(&self, ino: u32, offset: u64, size: usize) -> io::Result<Vec<u8>> {
        let mut file: ext2_file_t = ptr::null_mut();
        let retval = unsafe { ext2fs_file_open(self.fs, ino, 0, &mut file) };
        if retval != 0 || file.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to open file inode {}: {}", ino, retval),
            ));
        }

        let mut ret_got: u32 = 0;
        if offset > 0 {
            unsafe {
                ext2fs_file_llseek(file, offset as u64, 0, ptr::null_mut());

            }
        }

        let mut buf = vec![0u8; size];
        let read_res = unsafe {
            ext2fs_file_read(
                file,
                buf.as_mut_ptr() as *mut _,
                size as u32,
                &mut ret_got,
            )
        };

        unsafe {
            ext2fs_file_close(file);
        }

        if read_res != 0 {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to read file inode {}: {}", ino, read_res),
            ))
        } else {
            buf.truncate(ret_got as usize);
            Ok(buf)
        }
    }

    pub fn close(mut self) {
        if !self.fs.is_null() {
            unsafe {
                ext2fs_close(self.fs);
            }
            self.fs = ptr::null_mut();
        }
    }
}

impl Drop for Ext2FsHandle {
    fn drop(&mut self) {
        if !self.fs.is_null() {
            unsafe {
                ext2fs_close(self.fs);
            }
            self.fs = ptr::null_mut();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_existent_file_open_fails_gracefully() {
        let res = Ext2FsHandle::open("/non_existent_ext4_dev.img", true);
        assert!(res.is_err());
    }
}

