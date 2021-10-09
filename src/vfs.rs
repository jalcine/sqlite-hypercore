#![allow(non_snake_case)]
use crate::sqlite3;
use std::convert::AsMut;
use std::ffi::CString;
use std::mem;
use std::os::raw;

#[derive(Debug)]
pub enum AccessFlag {
    #[allow(unused)]
    Exists = 0,
    #[allow(unused)]
    ReadOnly = 1,
    #[allow(unused)]
    ReadWrite = 2,
}

pub trait VirtualFile {}

pub trait VirtualFilesystem<File: VirtualFile>: AsMut<Self> {
    fn get_vfs_name(&self) -> &CString;
    fn open(&self, path: &str, open_flags: &rusqlite::OpenFlags) -> Result<File, sqlite3::Error>
    where
        File: VirtualFile;
    fn delete(&mut self, path: &str, sync_to_system: bool) -> Result<(), sqlite3::Error>;
    fn access(&self, path: &str, access_flags: &[AccessFlag]) -> Result<(), sqlite3::Error>;
    fn full_pathname(&self, path: &str);

    /*
     * int (*xOpen)(sqlite3_vfs*, const char *zName, sqlite3_file*, int flags, int *pOutFlags);
     * int (*xDelete)(sqlite3_vfs*, const char *zName, int syncDir);
     * int (*xAccess)(sqlite3_vfs*, const char *zName, int flags, int *pResOut);
     * int (*xFullPathname)(sqlite3_vfs*, const char *zName, int nOut, char *zOut);
     * void *(*xDlOpen)(sqlite3_vfs*, const char *zFilename);
     * void (*xDlError)(sqlite3_vfs*, int nByte, char *zErrMsg);
     * void (*(*xDlSym)(sqlite3_vfs*,void*, const char *zSymbol))(void);
     * void (*xDlClose)(sqlite3_vfs*, void*);
     * int (*xRandomness)(sqlite3_vfs*, int nByte, char *zOut);
     * int (*xSleep)(sqlite3_vfs*, int microseconds);
     * int (*xCurrentTime)(sqlite3_vfs*, double*);
     * int (*xGetLastError)(sqlite3_vfs*, int, char *);
     * int (*xCurrentTimeInt64)(sqlite3_vfs*, sqlite3_int64*);
     * int (*xSetSystemCall)(sqlite3_vfs*, const char *zName, sqlite3_syscall_ptr);
     * sqlite3_syscall_ptr (*xGetSystemCall)(sqlite3_vfs*, const char *zName);
     * const char *(*xNextSystemCall)(sqlite3_vfs*, const char *zName);
     */
}

mod funcs {
    use super::{mem, raw, sqlite3};

    pub unsafe extern "C" fn open(
        _vfs: *mut sqlite3::sqlite3_vfs,
        zPath: *const raw::c_char,
        _file_ptr: *mut sqlite3::sqlite3_file,
        flags: raw::c_int,
        _pOutFlags: *mut raw::c_int,
    ) -> raw::c_int {
        log::trace!("Attempting to open a file {:?} with {:?}.", zPath, flags);
        sqlite3::SQLITE_OK as i32
    }

    pub unsafe extern "C" fn delete(
        _vfs: *mut sqlite3::sqlite3_vfs,
        zName: *const raw::c_char,
        _syncDir: raw::c_int,
    ) -> raw::c_int {
        log::trace!("Attempting to delete a file {:?}", zName);
        sqlite3::SQLITE_OK as i32
    }
    pub unsafe extern "C" fn access(
        _vfs: *mut sqlite3::sqlite3_vfs,
        zName: *const raw::c_char,
        flags: raw::c_int,
        _pResOut: *mut raw::c_int,
    ) -> raw::c_int {
        log::trace!(
            "Attempting to check of the file access for {:?} with {:?}",
            zName,
            flags
        );
        sqlite3::SQLITE_OK as i32
    }

    pub unsafe extern "C" fn full_pathname(
        _vfs: *mut sqlite3::sqlite3_vfs,
        name: *const raw::c_char,
        _nOut: raw::c_int,
        _zOut: *mut raw::c_char,
    ) -> raw::c_int {
        log::trace!("Resolving the full path file of {:?}", name);
        sqlite3::SQLITE_OK as i32
    }

    pub unsafe extern "C" fn dl_open(
        _vfs: *mut sqlite3::sqlite3_vfs,
        _zFilename: *const raw::c_char,
    ) -> *mut raw::c_void {
        log::trace!("Opening up the dylib.");
        mem::zeroed()
    }
    pub unsafe extern "C" fn dl_error(
        _vfs: *mut sqlite3::sqlite3_vfs,
        _nByte: raw::c_int,
        _zErrMsg: *mut raw::c_char,
    ) {
        log::trace!("Ran into an error with the dylib.");
    }

    pub unsafe extern "C" fn dl_close(_vfs: *mut sqlite3::sqlite3_vfs, _zWhat: *mut raw::c_void) {
        log::trace!("Closing out the dylib.");
    }
    pub unsafe extern "C" fn dl_sym(
        _vfs: *mut sqlite3::sqlite3_vfs,
        _zWhat: *mut raw::c_void,
        zSymbol: *const raw::c_char,
    ) -> Option<unsafe extern "C" fn()> {
        log::trace!("Resolving the symbol from the dylib of {:?}", zSymbol);
        None
    }
    pub unsafe extern "C" fn current_time(
        _vfs: *mut sqlite3::sqlite3_vfs,
        _timestamp: *mut raw::c_double,
    ) -> raw::c_int {
        log::trace!("Getting the current time right now");
        sqlite3::SQLITE_OK as i32
    }
    pub unsafe extern "C" fn current_time_int64(
        _vfs: *mut sqlite3::sqlite3_vfs,
        _timestamp: *mut sqlite3::sqlite3_int64,
    ) -> raw::c_int {
        log::trace!("Getting the current time right now buttt as a int64");
        sqlite3::SQLITE_OK as i32
    }
    pub unsafe extern "C" fn get_last_error(
        _vfs: *mut sqlite3::sqlite3_vfs,
        error_code: raw::c_int,
        error_something: *mut raw::c_schar,
    ) -> raw::c_int {
        log::trace!(
            "The last error found were {:?} / {:?}",
            error_code,
            error_something
        );
        sqlite3::SQLITE_OK as i32
    }
    pub unsafe extern "C" fn randomness(
        _vfs: *mut sqlite3::sqlite3_vfs,
        _nByte: raw::c_int,
        message: *mut raw::c_char,
    ) -> raw::c_int {
        log::trace!("Make something random for {:?}", message);
        sqlite3::SQLITE_OK as i32
    }
    pub unsafe extern "C" fn sleep(
        _vfs: *mut sqlite3::sqlite3_vfs,
        microseconds: raw::c_int,
    ) -> raw::c_int {
        log::trace!("They wanna sleep for {:?} microseconds", microseconds);
        sqlite3::SQLITE_OK as i32
    }

    pub unsafe extern "C" fn next_system_call(
        _vfs: *mut sqlite3::sqlite3_vfs,
        _sys_call_name: *const raw::c_char,
    ) -> *const raw::c_char {
        log::trace!("Okay so they want the next system call.");
        mem::zeroed()
    }

    // pub unsafe extern "C" fn get_system_call(
    //     vfs: *mut sqlite3::sqlite3_vfs,
    //     sys_call_name: *const raw::c_char,
    // ) -> sqlite3::sqlite3_syscall_ptr {
    //     log::trace!(
    //         "Okay so they want to get a system call named {:?}.",
    //         sys_call_name
    //     );
    //     mem::zeroed()
    // }

    // pub unsafe extern "C" fn set_system_call(
    //     vfs: *mut sqlite3::sqlite3_vfs,
    //     sys_call_name: *const raw::c_char,
    //     syscall_ptr: sqlite3::sqlite3_syscall_ptr,
    // ) -> raw::c_int {
    //     log::trace!(
    //         "Okay so they want to set a system call named {:?}.",
    //         sys_call_name
    //     );
    //     mem::zeroed()
    // }
}

pub fn register_for<File, FileSystem>(filesystem: &mut FileSystem) -> sqlite3::sqlite3_vfs
where
    File: VirtualFile,
    FileSystem: VirtualFilesystem<File>,
{
    // We need to put `filesystem` into `p_app_data` so we can extract
    // from the generic methods.
    let szOsFile = mem::size_of::<Box<FileSystem>>() as raw::c_int;

    let mut fs = sqlite3::sqlite3_vfs {
        iVersion: 2,
        zName: filesystem.get_vfs_name().as_ptr() as _,
        mxPathname: 512,
        pNext: std::ptr::null_mut(),
        pAppData: std::ptr::null_mut(),
        szOsFile,
        xOpen: Some(funcs::open),
        xDelete: Some(funcs::delete),
        xAccess: Some(funcs::access),
        xFullPathname: Some(funcs::full_pathname),
        xDlOpen: Some(funcs::dl_open),
        xDlError: Some(funcs::dl_error),
        xDlClose: Some(funcs::dl_close),
        xDlSym: Some(funcs::dl_sym),
        xCurrentTime: Some(funcs::current_time),
        xGetLastError: Some(funcs::get_last_error),
        xRandomness: Some(funcs::randomness),
        xSleep: Some(funcs::sleep),
        // xCurrentTimeInt64: Some(funcs::current_time_int64),
        // xGetSystemCall: Some(funcs::get_system_call),
        // xNextSystemCall: Some(funcs::next_system_call),
        // xSetSystemCall: Some(funcs::set_system_call),
    };
    fs.pAppData = unsafe { std::mem::transmute(filesystem) };
    fs
}
