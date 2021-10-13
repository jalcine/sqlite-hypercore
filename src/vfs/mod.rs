#![allow(non_snake_case)]
use rusqlite::ffi as sqlite3;
use std::cell::RefCell;
use std::ffi::CString;
use std::mem::{self, MaybeUninit};
use std::os::raw;
use std::ptr::{null_mut, NonNull};
use std::rc::Rc;

/// Represents the access level of a file.
// FIXME: Turn into a bitwise flag.
pub enum AccessFlag {
    #[allow(unused)]
    Exists = sqlite3::SQLITE_ACCESS_EXISTS as _,
    #[allow(unused)]
    ReadOnly = sqlite3::SQLITE_ACCESS_READ as _,
    #[allow(unused)]
    ReadWrite = sqlite3::SQLITE_ACCESS_READWRITE as _,
}

// FIXME: Turn into a bitwise flag.
pub enum LockFlag {
    None = sqlite3::SQLITE_LOCK_NONE as _,
    Shared = sqlite3::SQLITE_LOCK_SHARED as _,
    Reserved = sqlite3::SQLITE_LOCK_RESERVED as _,
    Pending = sqlite3::SQLITE_LOCK_PENDING as _,
    Exclusive = sqlite3::SQLITE_LOCK_EXCLUSIVE as _,
}

// NOTE: This is fixed to version 1 of sqlite3_file.
pub trait VirtualFile {
    // int (*xClose)(sqlite3_file*);
    fn close(&self) -> anyhow::Result<()>;

    // int (*xRead)(sqlite3_file*, void*, int iAmt, sqlite3_int64 iOfst);
    fn read(&self, amount: raw::c_int, offset: sqlite3::sqlite3_int64) -> anyhow::Result<Vec<u8>>;

    // int (*xWrite)(sqlite3_file*, const void*, int iAmt, sqlite3_int64 iOfst);
    fn write(
        &self,
        data: Vec<u8>,
        amount: raw::c_int,
        offset: sqlite3::sqlite3_int64,
    ) -> anyhow::Result<raw::c_int>;

    // int (*xTruncate)(sqlite3_file*, sqlite3_int64 size);
    fn truncate(&self, length: sqlite3::sqlite3_int64) -> anyhow::Result<()>;

    // int (*xSync)(sqlite3_file*, int flags);
    fn sync(&self, flags: raw::c_int) -> anyhow::Result<()>;

    // int (*xFileSize)(sqlite3_file*, sqlite3_int64 *pSize);
    fn size(&self) -> anyhow::Result<sqlite3::sqlite3_int64>;

    // int (*xLock)(sqlite3_file*, int);
    fn lock(&self, flag: LockFlag) -> anyhow::Result<()>;

    // int (*xUnlock)(sqlite3_file*, int);
    fn unlock(&self, flag: LockFlag) -> anyhow::Result<()>;

    // int (*xCheckReservedLock)(sqlite3_file*, int *pResOut);
    fn check_reserved_lock(&self) -> anyhow::Result<bool>;

    // int (*xFileControl)(sqlite3_file*, int op, void *pArg);
    fn file_control(&self, op: raw::c_int, structure: *const raw::c_void);

    // int (*xSectorSize)(sqlite3_file*);
    fn sector_size(&self) -> raw::c_int;

    // int (*xDeviceCharacteristics)(sqlite3_file*);
    // FIXME: Make a bitwise flag of the IO characteristics to use.
    fn device_characteristics(&self) -> Vec<raw::c_int>;
}

pub trait VirtualFilesystem {
    /// Called when SQLite is attempting to open a file on the system.
    fn open(
        &self,
        path: &str,
        open_flags: &rusqlite::OpenFlags,
    ) -> Result<Box<dyn VirtualFile>, sqlite3::ErrorCode>;

    /// Called when SQLite is attempting to delete a file on the system.
    fn delete(&mut self, path: &str, sync_to_system: bool) -> Result<(), sqlite3::ErrorCode>;

    /// Called when SQLite is attempting to determine access information about a file on the
    /// system.
    fn access(&self, path: &str, access_flags: &[AccessFlag]) -> Result<(), sqlite3::ErrorCode>;

    /// Called to obtain the full path name of the provided string from the filesystem.
    fn full_pathname(&self, path: &str) -> Result<String, sqlite3::ErrorCode>;
}

// Represents the C-visible functions that'd call back into Rust to handle work with the virtual
// filesystems we'd implement here.
mod funcs {
    use std::cell::RefCell;
    use std::ffi::{c_void, CStr};
    use std::mem::zeroed;
    use std::os::raw::{c_char, c_double, c_int, c_schar};
    use std::rc::Rc;

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
                *resolved_path_name = resolved_path.as_ptr() as _;
            }
            Err(_) => {
                *resolved_path_name = path_name as _;
            }
        }
        SQLITE_OK as _
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

    pub unsafe extern "C" fn open_file(
        ptr: *mut sqlite3_vfs,
        path_name: *const c_char,
        _file_ptr: *mut sqlite3_file,
        _flags: c_int,
        _output_flags: *mut c_int,
    ) -> c_int {
        let _ = env_logger::builder().is_test(true).try_init();
        let vfs_name = CStr::from_ptr((*ptr).zName);
        let path_name_str = CStr::from_ptr(path_name);

        log::trace!(
            "Attempting to open a file at {:?} via the {:?} VFS.",
            path_name_str,
            vfs_name
        );

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

#[repr(C)]
#[derive(Clone)]
pub struct Instance {
    ptr: sqlite3::sqlite3_vfs,
    fs: Rc<RefCell<dyn VirtualFilesystem>>,
    vfs_name: CString,
}

impl Instance {
    pub fn new(
        vfs_name: impl ToString,
        filesystem: Rc<RefCell<dyn VirtualFilesystem>>,
    ) -> anyhow::Result<Rc<RefCell<Self>>> {
        let vfs: sqlite3::sqlite3_vfs = unsafe { MaybeUninit::uninit().assume_init() };
        Ok(Rc::new(RefCell::new(Self {
            ptr: vfs,
            fs: Rc::clone(&filesystem),
            vfs_name: CString::new(vfs_name.to_string().into_bytes())?,
        })))
    }

    /// The name of the VFS.
    pub fn vfs_name(&self) -> Option<String> {
        CString::into_string(self.vfs_name.clone()).ok()
    }

    pub fn filesystem(&self) -> Rc<RefCell<dyn VirtualFilesystem>> {
        Rc::clone(&self.fs)
    }

    fn into_raw(instance_rc: Rc<RefCell<Self>>) -> *mut raw::c_void {
        Box::into_raw(Box::new(Rc::clone(&instance_rc))) as *mut raw::c_void
    }

    pub fn register(instance_rc: Rc<RefCell<Self>>, make_default: bool) -> anyhow::Result<()> {
        if !instance_rc.borrow().registered() {
            {
                let file_ptr_size = mem::size_of::<Box<dyn VirtualFile>>() as raw::c_int;
                let mut instance_mut = instance_rc.borrow_mut();
                instance_mut.ptr.iVersion = 1;
                instance_mut.ptr.mxPathname = 1024;
                instance_mut.ptr.zName = instance_mut.vfs_name.as_ptr() as _;
                instance_mut.ptr.szOsFile = file_ptr_size;
                instance_mut.ptr.pNext = std::ptr::null_mut();
                instance_mut.ptr.xOpen = Some(funcs::open_file);
                instance_mut.ptr.xFullPathname = Some(funcs::resolve_full_path_name);
                instance_mut.ptr.xAccess = Some(funcs::get_file_access);
                instance_mut.ptr.xDelete = Some(funcs::delete_file);
                instance_mut.ptr.xDlOpen = Some(funcs::dl_open);
                instance_mut.ptr.xDlError = Some(funcs::dl_error);
                instance_mut.ptr.xDlSym = Some(funcs::dl_sym);
                instance_mut.ptr.xDlClose = Some(funcs::dl_close);
                instance_mut.ptr.xRandomness = Some(funcs::randomness);
                instance_mut.ptr.xSleep = Some(funcs::sleep);
                instance_mut.ptr.xCurrentTime = Some(funcs::current_time);
                instance_mut.ptr.xGetLastError = Some(funcs::get_last_error);
                instance_mut.ptr.pAppData = Self::into_raw(Rc::clone(&instance_rc));
            }
            log::info!(
                "Attempting to register VFS for {:?}",
                instance_rc.borrow().vfs_name
            );

            // FIXME: Look into leaning on rusqlite to handle error reporting from SQLite.

            let register_result = unsafe {
                sqlite3::sqlite3_vfs_register(
                    &mut instance_rc.borrow_mut().ptr,
                    make_default as raw::c_int,
                )
            };

            if register_result == sqlite3::SQLITE_OK as _ {
                log::info!(
                    "Registered {:?} into the SQLite VFS index.",
                    instance_rc.borrow().vfs_name
                );
                Ok(())
            } else {
                log::error!(
                    "Failed to register {:?} into the SQLite VFS index (code: {}).",
                    instance_rc.borrow().vfs_name,
                    register_result,
                );
                Err(anyhow::anyhow!("Failed to register VFS"))
            }
        } else {
            Ok(())
        }
    }

    /// Checks to see if the `VirtualFilesystem` held by this instance has been registered.
    pub fn registered(&self) -> bool {
        NonNull::new(unsafe { sqlite3::sqlite3_vfs_find(self.vfs_name.as_ptr() as _) }).is_some()
    }
}

#[cfg(test)]
mod test {
    use log::kv::ToValue;

    use super::*;
    use std::ops::Deref;

    struct MockFile {}

    impl VirtualFile for MockFile {
        fn close(&self) -> anyhow::Result<()> {
            Ok(())
        }

        fn read(
            &self,
            amount: raw::c_int,
            offset: sqlite3::sqlite3_int64,
        ) -> anyhow::Result<Vec<u8>> {
            todo!()
        }

        fn write(
            &self,
            data: Vec<u8>,
            amount: raw::c_int,
            offset: sqlite3::sqlite3_int64,
        ) -> anyhow::Result<raw::c_int> {
            todo!()
        }

        fn truncate(&self, length: sqlite3::sqlite3_int64) -> anyhow::Result<()> {
            todo!()
        }

        fn sync(&self, flags: raw::c_int) -> anyhow::Result<()> {
            todo!()
        }

        fn size(&self) -> anyhow::Result<sqlite3::sqlite3_int64> {
            todo!()
        }

        fn lock(&self, flag: LockFlag) -> anyhow::Result<()> {
            todo!()
        }

        fn unlock(&self, flag: LockFlag) -> anyhow::Result<()> {
            todo!()
        }

        fn check_reserved_lock(&self) -> anyhow::Result<bool> {
            todo!()
        }

        fn file_control(&self, op: raw::c_int, structure: *const raw::c_void) {
            unimplemented!()
        }

        fn sector_size(&self) -> raw::c_int {
            todo!()
        }

        fn device_characteristics(&self) -> Vec<raw::c_int> {
            todo!()
        }
    }

    struct MockFilesystem {}

    impl VirtualFilesystem for MockFilesystem {
        fn access(
            &self,
            _path: &str,
            _access_flags: &[AccessFlag],
        ) -> Result<(), sqlite3::ErrorCode> {
            unimplemented!()
        }

        fn delete(&mut self, _path: &str, _sync_to_system: bool) -> Result<(), sqlite3::ErrorCode> {
            Ok(())
        }

        fn full_pathname(&self, path: &str) -> Result<String, sqlite3::ErrorCode> {
            Ok(path.to_string())
        }

        fn open(
            &self,
            path: &str,
            _open_flags: &rusqlite::OpenFlags,
        ) -> Result<Box<dyn VirtualFile>, sqlite3::ErrorCode> {
            log::trace!(
                "Attempting to look up the file {:?} in the mock system.",
                path
            );

            if path == "mock-system.db" {
                log::trace!("Used the expected mock file name.");
                Ok(Box::new(MockFile {}))
            } else {
                log::trace!("Didn't recognize the name {:?}; failing out.", path);
                Err(sqlite3::ErrorCode::NotFound)
            }
        }
    }

    #[test]
    fn registers_filesystem() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        let mock_fs = Rc::new(RefCell::new(MockFilesystem {}));
        let inst = Instance::new("mock-init", mock_fs)?;
        assert!(Instance::register(Rc::clone(&inst), false).is_ok());
        assert!(inst.deref().borrow().registered());
        Ok(())
    }

    #[test]
    fn open_database_connection() -> anyhow::Result<()> {
        let _ = env_logger::builder().is_test(true).try_init();
        let mock_fs = Rc::new(RefCell::new(MockFilesystem {}));
        let inst = Instance::new("mock-connect", mock_fs)?;
        assert!(Instance::register(Rc::clone(&inst), false).is_ok());
        assert!(inst.deref().borrow().registered());

        log::info!("Connecting to the database...");
        let conn = rusqlite::Connection::open_with_flags_and_vfs(
            "mock-system.db",
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE,
            &inst.deref().borrow().vfs_name().unwrap(),
        )?;

        log::info!("Attempting to load schema.");
        assert!(conn
            .execute(
                r#"
            CREATE TABLE IF NOT EXISTS sample(
                name TEXT
            );
        "#,
                rusqlite::NO_PARAMS,
            )
            .is_ok());

        log::info!("Disconnecting");
        drop(conn);
        Ok(())
    }
}
