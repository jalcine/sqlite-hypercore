use super::{sqlite3, LockFlag};
use std::cell::RefCell;
use std::os::raw;
use std::rc::Rc;

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

#[repr(C)]
#[derive(Clone)]
pub struct WrappedFile {
    pub ptr: Option<sqlite3::sqlite3_file>,
    methods: Option<sqlite3::sqlite3_io_methods>,
    handle: Rc<RefCell<dyn VirtualFile>>,
}

impl WrappedFile {
    fn bind(&mut self) {
        self.methods = Some(sqlite3::sqlite3_io_methods {
            iVersion: 1,
            xClose: todo!(),
            xRead: todo!(),
            xWrite: todo!(),
            xTruncate: todo!(),
            xSync: todo!(),
            xFileSize: todo!(),
            xLock: todo!(),
            xUnlock: todo!(),
            xCheckReservedLock: todo!(),
            xFileControl: todo!(),
            xSectorSize: todo!(),
            xDeviceCharacteristics: todo!(),
            xShmMap: todo!(),
            xShmLock: todo!(),
            xShmBarrier: todo!(),
            xShmUnmap: todo!(),
            xFetch: todo!(),
            xUnfetch: todo!(),
        });

        self.ptr = Some(sqlite3::sqlite3_file {
            pMethods: self.methods.as_ref().unwrap(),
        });
    }
    pub fn wrap(file_ptr: Rc<RefCell<dyn VirtualFile>>) -> Self {
        let mut item = Self {
            ptr: None,
            methods: None,
            handle: Rc::clone(&file_ptr),
        };
        item.bind();

        item
    }
}
