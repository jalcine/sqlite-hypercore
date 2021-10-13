use super::{file::WrappedFile, sqlite3, AccessFlag, Instance};
use std::{mem, os::raw};

pub trait VirtualFilesystem {
    /// Called when SQLite is attempting to open a file on the system.
    fn open(
        &self,
        path: &str,
        open_flags: &rusqlite::OpenFlags,
    ) -> Result<Box<WrappedFile>, sqlite3::ErrorCode>;

    /// Called when SQLite is attempting to delete a file on the system.
    fn delete(&mut self, path: &str, sync_to_system: bool) -> Result<(), sqlite3::ErrorCode>;

    /// Called when SQLite is attempting to determine access information about a file on the
    /// system.
    fn access(&self, path: &str, access_flags: &[AccessFlag]) -> Result<(), sqlite3::ErrorCode>;

    /// Called to obtain the full path name of the provided string from the filesystem.
    fn full_pathname(&self, path: &str) -> Result<String, sqlite3::ErrorCode>;
}

mod funcs {
    use std::cell::RefCell;
    use std::ffi::{c_void, CStr};
    use std::mem::zeroed;
    use std::os::raw::{c_char, c_double, c_int, c_schar};
    use std::rc::Rc;

    use rusqlite::OpenFlags;

    use super::{
        sqlite3::{sqlite3_file, sqlite3_vfs, SQLITE_IOERR_CLOSE, SQLITE_OK},
        Instance,
    };

    unsafe fn extract_instance(vfs_ptr: *mut sqlite3_vfs) -> Option<Box<Rc<RefCell<Instance>>>> {
        let app_data = (*vfs_ptr).pAppData;

        if app_data.is_null() {
            log::error!("Couldn't find any reference to the Instance in this VFS pointer.");
            None
        } else {
            let b = Box::from_raw(app_data as *mut Rc<RefCell<Instance>>);
            assert!(!b.as_ptr().is_null());
            Some(b)
        }
    }

    // FIXME: Actually use and correct the path using `null_terminator`.
    pub unsafe extern "C" fn resolve_full_path_name(
        ptr: *mut sqlite3_vfs,
        path_name: *const c_char,
        _null_terminator: c_int,
        resolved_path_name: *mut c_char,
    ) -> c_int {
        let _ = env_logger::builder().is_test(true).try_init();
        let vfs_name = CStr::from_ptr((*ptr).zName);
        let path_name_str = CStr::from_ptr(path_name);
        log::trace!(
            "Attempting to resolve the full path name of {:?} from {:?}.",
            path_name_str,
            vfs_name
        );

        let vfs_inst = extract_instance(ptr).expect("Could not find the instance.");

        match vfs_inst.borrow().filesystem().borrow().full_pathname(
            path_name_str
                .to_str()
                .expect("Invalid pointer for the string representing the path of the file name."),
        ) {
            Ok(resolved_path) => {
                log::trace!(
                    "Resolved {:?} as the full path of {:?} from the {:?} VFS.",
                    resolved_path,
                    path_name_str,
                    vfs_name
                );
                let resolved_path_name_str = CStr::from_ptr(resolved_path.as_ptr() as _);
                *resolved_path_name = resolved_path_name_str.as_ptr() as _;
            }
            Err(_) => {
                *resolved_path_name = path_name as _;
            }
        }
        SQLITE_OK as _
    }

    pub unsafe extern "C" fn open_file(
        ptr: *mut sqlite3_vfs,
        path_name: *const c_char,
        file_ptr: *mut sqlite3_file,
        open_flags_bits: c_int,
        output_flags: *mut c_int,
    ) -> c_int {
        let _ = env_logger::builder().is_test(true).try_init();
        let vfs_name = CStr::from_ptr((*ptr).zName);
        let path_name_str = CStr::from_ptr(path_name);
        let open_flags = OpenFlags::from_bits_truncate(open_flags_bits);

        log::trace!(
            "Attempting to open a file at {:?} via the {:?} VFS with the flags {:?}.",
            path_name_str,
            vfs_name,
            open_flags
        );

        let vfs_inst = extract_instance(ptr).expect("Could not find the instance.");

        let result = match vfs_inst.borrow().filesystem().borrow().open(
            path_name_str
                .to_str()
                .expect("Failed to craft string from pointer."),
            &open_flags,
        ) {
            Ok(file) => {
                log::trace!(
                    "The file {:?} was opened with {:?} as flags.",
                    path_name_str,
                    open_flags
                );
                *file_ptr = file.ptr.unwrap();
                SQLITE_OK as _
            }
            Err(code) => {
                log::error!(
                    "Could not open the file {:?}; error code {:?}",
                    path_name_str,
                    code
                );
                code as _
            }
        };
        result
    }
    pub unsafe extern "C" fn delete_file(
        _ptr: *mut sqlite3_vfs,
        _path_name: *const c_char,
        _immediate: c_int,
    ) -> c_int {
        let _ = env_logger::builder().is_test(true).try_init();
        log::trace!("Attempting to delete a file.");
        SQLITE_IOERR_CLOSE as _
    }
    pub unsafe extern "C" fn get_file_access(
        _ptr: *mut sqlite3_vfs,
        _path_name: *const c_char,
        _flags: c_int,
        _resolved_access_flags: *mut c_int,
    ) -> c_int {
        let _ = env_logger::builder().is_test(true).try_init();
        log::trace!("Attempting to get file access info.");
        SQLITE_IOERR_CLOSE as _
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
        SQLITE_OK as _
    }
    pub unsafe extern "C" fn get_last_error(
        _vfs: *mut sqlite3_vfs,
        error_code: c_int,
        error_something: *mut c_schar,
    ) -> c_int {
        let error_str = if !error_something.is_null() {
            CStr::from_ptr(error_something)
                .to_str()
                .unwrap_or("Unexpected error.")
                .to_string()
        } else {
            "Unexpected error - no error message provided.".to_string()
        };
        log::trace!("The last error found was {:?}: {:?}", error_code, error_str);
        SQLITE_OK as _
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

pub fn bind(vfs: &mut sqlite3::sqlite3_vfs) {
    let file_ptr_size = mem::size_of::<Box<dyn super::File>>() as raw::c_int;
    vfs.iVersion = 1;
    vfs.mxPathname = 1024;
    vfs.pNext = std::ptr::null_mut();
    vfs.szOsFile = file_ptr_size;
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
}
