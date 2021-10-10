#![allow(non_snake_case)]
use rusqlite::ffi::{self as sqlite3};
use std::cell::RefCell;
use std::ffi::{c_void, CString};
use std::mem;
use std::ops::Deref;
use std::os::raw;
use std::ptr::NonNull;
use std::rc::Rc;

pub mod hyper;

/// Represents the access level of a file.
// FIXME: Alias to the ones used by SQLite itself.
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

pub trait VirtualFilesystem {
    /// Called when SQLite is attempting to open a file on the system.
    fn open(
        &self,
        path: &str,
        open_flags: &rusqlite::OpenFlags,
    ) -> Result<Box<dyn VirtualFile>, sqlite3::Error>;

    /// Called when SQLite is attempting to delete a file on the system.
    fn delete(&mut self, path: &str, sync_to_system: bool) -> Result<(), sqlite3::Error>;

    /// Called when SQLite is attempting to determine access information about a file on the
    /// system.
    fn access(&self, path: &str, access_flags: &[AccessFlag]) -> Result<(), sqlite3::Error>;

    /// Called to obtain the full path name of the provided string from the filesystem.
    fn full_pathname(&self, path: &str) -> Result<String, sqlite3::Error>;
}

// mod funcs {
//     use super::sqlite3;
//     use std::mem;
//     use std::os::raw;

//     pub unsafe extern "C" fn open(
//         _vfs: *mut sqlite3::sqlite3_vfs,
//         path: *const raw::c_char,
//         _file: *mut sqlite3::sqlite3_file,
//         flags: raw::c_int,
//         _out_flags: *mut raw::c_int,
//     ) -> raw::c_int {
//         log::trace!("Attempting to open a file {:?} with {:?}.", path, flags);
//         // FIXME: Get the containing object by the VirtualFilesystem.

//         sqlite3::SQLITE_OK as i32
//     }

//     pub unsafe extern "C" fn delete(
//         _vfs: *mut sqlite3::sqlite3_vfs,
//         name: *const raw::c_char,
//         _sync_to_disk: raw::c_int,
//     ) -> raw::c_int {
//         log::trace!("Attempting to delete a file {:?}", name);
//         sqlite3::SQLITE_OK as i32
//     }
//     pub unsafe extern "C" fn access(
//         _vfs: *mut sqlite3::sqlite3_vfs,
//         name: *const raw::c_char,
//         flags: raw::c_int,
//         _result: *mut raw::c_int,
//     ) -> raw::c_int {
//         log::trace!(
//             "Attempting to check of the file access for {:?} with {:?}",
//             name,
//             flags
//         );
//         sqlite3::SQLITE_OK as i32
//     }

//     pub unsafe extern "C" fn full_pathname(
//         _vfs: *mut sqlite3::sqlite3_vfs,
//         name: *const raw::c_char,
//         _out: raw::c_int,
//         _expanded_path_name: *mut raw::c_char,
//     ) -> raw::c_int {
//         log::trace!("Resolving the full path file of {:?}", name);
//         sqlite3::SQLITE_OK as i32
//     }

//     pub unsafe extern "C" fn current_time(
//         _vfs: *mut sqlite3::sqlite3_vfs,
//         _resulting_timestamp: *mut raw::c_double,
//     ) -> raw::c_int {
//         log::trace!("Getting the current time right now");
//         sqlite3::SQLITE_OK as i32
//     }
//     // pub unsafe extern "C" fn current_time_int64(
//     //     _vfs: *mut sqlite3::sqlite3_vfs,
//     //     _resulting_timestamp: *mut sqlite3::sqlite3_int64,
//     // ) -> raw::c_int {
//     //     log::trace!("Getting the current time right now buttt as a int64");
//     //     sqlite3::SQLITE_OK as i32
//     // }
//     pub unsafe extern "C" fn get_last_error(
//         _vfs: *mut sqlite3::sqlite3_vfs,
//         error_code: raw::c_int,
//         error_something: *mut raw::c_schar,
//     ) -> raw::c_int {
//         log::trace!(
//             "The last error found were {:?} / {:?}",
//             error_code,
//             error_something
//         );
//         sqlite3::SQLITE_OK as i32
//     }
//     pub unsafe extern "C" fn randomness(
//         _vfs: *mut sqlite3::sqlite3_vfs,
//         size_of_random_bytes: raw::c_int,
//         message: *mut raw::c_char,
//     ) -> raw::c_int {
//         log::trace!(
//             "Make something random for {:?} of {} bytes",
//             message,
//             size_of_random_bytes
//         );
//         sqlite3::SQLITE_OK as i32
//     }
//     pub unsafe extern "C" fn sleep(
//         _vfs: *mut sqlite3::sqlite3_vfs,
//         microseconds: raw::c_int,
//     ) -> raw::c_int {
//         log::trace!("They wanna sleep for {:?} microseconds", microseconds);
//         sqlite3::SQLITE_OK as i32
//     }

//     // pub unsafe extern "C" fn next_system_call(
//     //     _vfs: *mut sqlite3::sqlite3_vfs,
//     //     _sys_call_name: *const raw::c_char,
//     // ) -> *const raw::c_char {
//     //     log::trace!("Okay so they want the next system call.");
//     //     mem::zeroed()
//     // }

//     // pub unsafe extern "C" fn get_system_call(
//     //     vfs: *mut sqlite3::sqlite3_vfs,
//     //     sys_call_name: *const raw::c_char,
//     // ) -> sqlite3::sqlite3_syscall_ptr {
//     //     log::trace!(
//     //         "Okay so they want to get a system call named {:?}.",
//     //         sys_call_name
//     //     );
//     //     mem::zeroed()
//     // }

//     // pub unsafe extern "C" fn set_system_call(
//     //     vfs: *mut sqlite3::sqlite3_vfs,
//     //     sys_call_name: *const raw::c_char,
//     //     syscall_ptr: sqlite3::sqlite3_syscall_ptr,
//     // ) -> raw::c_int {
//     //     log::trace!(
//     //         "Okay so they want to set a system call named {:?}.",
//     //         sys_call_name
//     //     );
//     //     mem::zeroed()
//     // }
// }

pub struct Instance {
    name: CString,
    ptr: std::ptr::NonNull<sqlite3::sqlite3_vfs>,
}

impl Instance {
    pub fn register(
        vfs_name: impl Into<Vec<u8>>,
        _filesystem: impl VirtualFilesystem,
        make_default: bool,
    ) -> anyhow::Result<Rc<RefCell<Self>>> {
        let mut vfs_ptr: mem::MaybeUninit<sqlite3::sqlite3_vfs> = mem::MaybeUninit::uninit();
        let inst = Rc::new(RefCell::new(Self {
            name: CString::new(vfs_name)?,
            ptr: std::ptr::NonNull::new(vfs_ptr.as_mut_ptr())
                .ok_or(anyhow::anyhow!("Failed to allocate pointer for SQLite VFS"))?,
        }));

        let file_ptr_size = mem::size_of::<Box<dyn VirtualFile>>() as raw::c_int;

        let mut vfs = unsafe { inst.deref().borrow_mut().ptr.as_mut() };

        vfs.iVersion = 1;
        vfs.mxPathname = 1024;
        vfs.zName = inst.deref().borrow().name.as_ptr() as _;
        vfs.szOsFile = file_ptr_size;

        vfs.pNext = std::ptr::null_mut();
        // FIXME: We need to set this to be `inst` somehow.
        vfs.pAppData = inst.as_ptr() as *mut c_void;

        vfs.xOpen = Some(funcs::open_file);
        vfs.xFullPathname = Some(funcs::resolve_full_path_name);
        vfs.xAccess = Some(funcs::get_file_access);
        vfs.xDelete = Some(funcs::delete_file);
        vfs.xDlOpen = Some(funcs::dl_open);
        vfs.xDlError = Some(funcs::dl_error);
        vfs.xDlSym = Some(funcs::dl_sym);
        vfs.xDlClose = Some(funcs::dl_close);
        vfs.xRandomness = Some(funcs::randomness);
        vfs.xSleep = Some(funcs::sleep);
        vfs.xCurrentTime = Some(funcs::current_time);
        vfs.xGetLastError = Some(funcs::get_last_error);

        log::info!(
            "Attempting to register VFS for {:?}",
            inst.deref().borrow().name.clone()
        );

        let register_result =
            unsafe { sqlite3::sqlite3_vfs_register(vfs, make_default as raw::c_int) };

        if register_result == sqlite3::SQLITE_OK as _ {
            log::info!(
                "Registered {:?} into the SQLite VFS index.",
                inst.deref().borrow().name
            );
            Ok(inst)
        } else {
            log::error!(
                "Failed to register {:?} into the SQLite VFS index (code: {}).",
                inst.deref().borrow().name,
                register_result,
            );
            Err(anyhow::anyhow!("Failed to register VFS"))
        }
    }

    pub fn is_registered(&self) -> bool {
        NonNull::new(unsafe { sqlite3::sqlite3_vfs_find(self.name.as_ptr()) }).is_some()
    }

    pub fn vfs_name(&self) -> anyhow::Result<&str> {
        self.name.to_str().map_err(|e| anyhow::anyhow!(e))
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        log::debug!("Dropping the VFS named {:?}", self.name.clone());
        let unregister_result = unsafe { sqlite3::sqlite3_vfs_unregister(self.ptr.as_mut()) };
        if unregister_result == sqlite3::SQLITE_OK as _ {
            log::debug!("Successfully unregistered VFS {:?}", self.name.clone());
        }
        log::debug!(
            "The VFS dropping of {:?} resulted in {:?}",
            self.name.clone(),
            unregister_result
        );
    }
}

mod funcs {
    use std::ffi::c_void;
    use std::mem::zeroed;
    use std::os::raw::{c_char, c_double, c_int, c_schar};

    use rusqlite::ffi::{sqlite3_file, sqlite3_vfs, SQLITE_OK};

    pub unsafe extern "C" fn resolve_full_path_name(
        ptr: *mut sqlite3_vfs,
        path_name: *const c_char,
        null_terminator: c_int,
        resolved_path_name: *mut c_char,
    ) -> c_int {
        let _ = env_logger::builder().is_test(true).try_init();
        log::trace!("Attempting to resolve the full path name.");
        unimplemented!()
    }
    pub unsafe extern "C" fn delete_file(
        ptr: *mut sqlite3_vfs,
        path_name: *const c_char,
        immediate: c_int,
    ) -> c_int {
        let _ = env_logger::builder().is_test(true).try_init();
        log::trace!("Attempting to delete a file.");
        unimplemented!()
    }
    pub unsafe extern "C" fn get_file_access(
        ptr: *mut sqlite3_vfs,
        path_name: *const c_char,
        flags: c_int,
        resolved_access_flags: *mut c_int,
    ) -> c_int {
        let _ = env_logger::builder().is_test(true).try_init();
        log::trace!("Attempting to get file access info.");
        unimplemented!()
    }

    pub unsafe extern "C" fn open_file(
        ptr: *mut sqlite3_vfs,
        name: *const c_char,
        file_ptr: *mut sqlite3_file,
        flags: c_int,
        output_flags: *mut c_int,
    ) -> c_int {
        let _ = env_logger::builder().is_test(true).try_init();
        log::trace!("Attempting to open a file.");
        unimplemented!()
    }
    pub unsafe extern "C" fn dl_open(
        _vfs: *mut sqlite3_vfs,
        file_name: *const c_char,
    ) -> *mut c_void {
        log::trace!("Opening up the dylib at {:?}.", file_name);
        zeroed()
    }
    pub unsafe extern "C" fn dl_error(
        _vfs: *mut sqlite3_vfs,
        _error_code: c_int,
        _error_message: *mut c_char,
    ) {
        log::trace!("Ran into an error with the dylib.");
    }

    pub unsafe extern "C" fn dl_close(_vfs: *mut sqlite3_vfs, _arg: *mut c_void) {
        log::trace!("Closing out the dylib.");
    }
    pub unsafe extern "C" fn dl_sym(
        _vfs: *mut sqlite3_vfs,
        _arg: *mut c_void,
        symbol_name: *const c_char,
    ) -> Option<unsafe extern "C" fn(*mut sqlite3_vfs, *mut c_void, *const c_char)> {
        log::trace!("Resolving the symbol from the dylib of {:?}", symbol_name);
        None
    }
    pub unsafe extern "C" fn current_time(
        _vfs: *mut sqlite3_vfs,
        _resulting_timestamp: *mut c_double,
    ) -> c_int {
        log::trace!("Getting the current time right now");
        SQLITE_OK as i32
    }
    pub unsafe extern "C" fn get_last_error(
        _vfs: *mut sqlite3_vfs,
        error_code: c_int,
        error_something: *mut c_schar,
    ) -> c_int {
        log::trace!(
            "The last error found were {:?} / {:?}",
            error_code,
            error_something
        );
        SQLITE_OK as i32
    }
    pub unsafe extern "C" fn randomness(
        _vfs: *mut sqlite3_vfs,
        size_of_random_bytes: c_int,
        message: *mut c_char,
    ) -> c_int {
        log::trace!(
            "Make something random for {:?} of {} bytes",
            message,
            size_of_random_bytes
        );
        SQLITE_OK as i32
    }
    pub unsafe extern "C" fn sleep(_vfs: *mut sqlite3_vfs, microseconds: c_int) -> c_int {
        log::trace!("They wanna sleep for {:?} microseconds", microseconds);
        SQLITE_OK as i32
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct MockFilesystem {}
    impl VirtualFilesystem for MockFilesystem {
        fn open(
            &self,
            _path: &str,
            _open_flags: &rusqlite::OpenFlags,
        ) -> Result<Box<dyn VirtualFile>, sqlite3::Error> {
            unimplemented!()
        }

        fn delete(&mut self, _path: &str, _sync_to_system: bool) -> Result<(), sqlite3::Error> {
            unimplemented!()
        }

        fn access(&self, _path: &str, _access_flags: &[AccessFlag]) -> Result<(), sqlite3::Error> {
            unimplemented!()
        }
        fn full_pathname(&self, _path: &str) -> Result<String, sqlite3::Error> {
            unimplemented!()
        }
    }

    #[test]
    fn registers_mock_system() {
        let _ = env_logger::builder().is_test(true).try_init();
        let mock_fs = MockFilesystem {};
        let inst_result = Instance::register("mock", mock_fs, false);

        assert!(inst_result.is_ok());
        let inst = inst_result.unwrap();

        assert!(inst.deref().borrow().is_registered());

        drop(inst)
    }

    #[test]
    fn open_connection_using_mock_vfs() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        let mock_fs = MockFilesystem {};
        let inst = Instance::register("mock2", mock_fs, false)?;
        assert!(inst.deref().borrow().is_registered());

        log::info!("Connecting to the database...");
        let conn = rusqlite::Connection::open_with_flags_and_vfs(
            "mock-system.db",
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE,
            inst.deref().borrow().vfs_name()?,
        )?;

        log::info!("Attempting to load schema.");
        assert!(conn
            .execute(
                r#"
            CREATE TABLE IF NOT EXISTS sample(
                name TEXT
            );
        "#,
                [],
            )
            .is_ok());

        log::info!("Disconnecting");
        drop(conn);
        Ok(())
    }
}
