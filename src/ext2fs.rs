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

fn ext2_error_string(retval: i64, path: &Path) -> String {
    unsafe {
        initialize_ext2_error_table();
    }

    let raw_msg = unsafe {
        let msg_ptr = error_message(retval as _);
        if !msg_ptr.is_null() {
            if let Ok(cstr) = std::ffi::CStr::from_ptr(msg_ptr).to_str() {
                cstr.to_string()
            } else {
                format!("Error code {}", retval)
            }
        } else {
            format!("Error code {}", retval)
        }
    };

    let path_str = path.display().to_string();
    if retval == 2133571347 || raw_msg.contains("magic") || raw_msg.contains("ext2 19") {
        if path_str.starts_with("/dev/disk") && !path_str.contains('s') {
            let suggested = format!("{}s2", path_str.replace("/dev/disk", "/dev/rdisk"));
            return format!(
                "Bad magic number in superblock! You provided whole physical disk '{}' (which contains partition tables, not an ext4 filesystem directly).\n\n==> PLEASE USE THE PARTITION DEVICE INSTEAD: 'sudo alpaca-extfs {} /Volumes/Ext4Drive'",
                path_str, suggested
            );
        }
    }

    format!("{} (code: {})", raw_msg, retval)
}


impl Ext2FsHandle {
    pub fn open<P: AsRef<Path>>(path: P, read_only: bool) -> io::Result<Self> {
        let p_ref = path.as_ref();
        let c_path = CString::new(p_ref.as_os_str().as_bytes())
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
                format!("Failed to open ext4 filesystem: {}", ext2_error_string(retval as i64, p_ref)),
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

    pub fn create_file(
        &self,
        parent_ino: u32,
        name: &str,
        mode: u16,
        uid: u32,
        gid: u32,
    ) -> io::Result<u32> {
        let mut new_ino: ext2_ino_t = 0;
        let retval = unsafe {
            ext2fs_new_inode(
                self.fs,
                parent_ino,
                mode as i32,
                ptr::null_mut(),
                &mut new_ino,
            )
        };
        if retval != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to allocate inode: {}", retval),
            ));
        }

        unsafe {
            ext2fs_inode_alloc_stats2(self.fs, new_ino, 1, 0);
        }

        let mut inode: ext2_inode = unsafe { std::mem::zeroed() };
        inode.i_mode = mode | 0o100000; // regular file
        inode.i_links_count = 1;
        inode.i_uid = uid as u16;
        inode.i_gid = gid as u16;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;
        inode.i_atime = now;
        inode.i_mtime = now;
        inode.i_ctime = now;

        let retval = unsafe { ext2fs_write_inode(self.fs, new_ino, &mut inode) };
        if retval != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to write new inode: {}", retval),
            ));
        }

        let c_name = CString::new(name)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let retval = unsafe {
            ext2fs_link(
                self.fs,
                parent_ino,
                c_name.as_ptr(),
                new_ino,
                EXT2_FT_REG_FILE as i32,
            )
        };
        if retval != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to link file: {}", retval),
            ));
        }

        self.flush()?;
        Ok(new_ino)
    }

    pub fn write_file(&self, ino: u32, offset: u64, data: &[u8]) -> io::Result<usize> {
        let mut file: ext2_file_t = ptr::null_mut();
        let retval = unsafe { ext2fs_file_open(self.fs, ino, EXT2_FILE_WRITE as i32, &mut file) };
        if retval != 0 || file.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to open file for writing: {}", retval),
            ));
        }

        if offset > 0 {
            unsafe {
                ext2fs_file_llseek(file, offset as u64, 0, ptr::null_mut());
            }
        }

        let mut written: u32 = 0;
        let write_res = unsafe {
            ext2fs_file_write(
                file,
                data.as_ptr() as *const _,
                data.len() as u32,
                &mut written,
            )
        };

        unsafe {
            ext2fs_file_close(file);
        }

        if write_res != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to write file data: {}", write_res),
            ));
        }

        self.flush()?;
        Ok(written as usize)
    }

    pub fn unlink(&self, parent_ino: u32, name: &str) -> io::Result<()> {
        let c_name = CString::new(name)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let retval = unsafe { ext2fs_unlink(self.fs, parent_ino, c_name.as_ptr(), 0, 0) };
        if retval != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to unlink {}: {}", name, retval),
            ));
        }
        self.flush()?;
        Ok(())
    }

    pub fn mkdir(
        &self,
        parent_ino: u32,
        name: &str,
        mode: u16,
        _uid: u32,
        _gid: u32,
    ) -> io::Result<u32> {
        let mut new_ino: ext2_ino_t = 0;
        let retval = unsafe {
            ext2fs_new_inode(
                self.fs,
                parent_ino,
                (mode | 0o040000) as i32,
                ptr::null_mut(),
                &mut new_ino,
            )
        };
        if retval != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to allocate dir inode: {}", retval),
            ));
        }

        let c_name = CString::new(name)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        let retval = unsafe { ext2fs_mkdir(self.fs, parent_ino, new_ino, c_name.as_ptr()) };
        if retval != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to mkdir {}: {}", name, retval),
            ));
        }

        self.flush()?;
        Ok(new_ino)
    }

    pub fn flush(&self) -> io::Result<()> {
        let retval = unsafe { ext2fs_flush(self.fs) };
        if retval != 0 {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to flush ext4 filesystem: {}", retval),
            ))
        } else {
            Ok(())
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

