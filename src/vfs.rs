pub use libsqlite3_sys as sqlite3;
use std::cell::RefCell;
use std::ffi::CString;
use std::mem;
use std::ops::Deref;
use std::os::raw::c_int;
use std::rc::Rc;

/// This _should_ be where all of the logic for working with Hypercore and SQLite happen.
/// Reference: https://sqlite.org/c3ref/vfs.html
struct Vfs {
    reference: Box<sqlite3::sqlite3_vfs>,
}

/// This would also represent a "file". I think. It's more like a file handle.
/// Reference: https://sqlite.org/c3ref/file.html
struct File {
    io_methods: Rc<RefCell<IoMethods>>,
    reference: Box<sqlite3::sqlite3_file>,
}

/// And this represents the operations one could take on a file.
/// Reference: https://sqlite.org/c3ref/io_methods.html
struct IoMethods {
    reference: Box<sqlite3::sqlite3_io_methods>,
}

impl IoMethods {
    pub fn new() -> RefCell<Self> {
        RefCell::new(Self {
            reference: Box::new(sqlite3::sqlite3_io_methods {
                iVersion: 3,
                xClose: None,
                xRead: None,
                xWrite: None,
                xTruncate: None,
                xSync: None,
                xFileSize: None,
                xLock: None,
                xUnlock: None,
                xCheckReservedLock: None,
                xFileControl: None,
                xSectorSize: None,
                xDeviceCharacteristics: None,
                xFetch: None,
                xShmBarrier: None,
                xShmLock: None,
                xShmMap: None,
                xShmUnmap: None,
                xUnfetch: None,
                /*
                FIXME: Define the following functions.
                   int (*xClose)(sqlite3_file*);
                   int (*xRead)(sqlite3_file*, void*, int iAmt, sqlite3_int64 iOfst);
                   int (*xWrite)(sqlite3_file*, const void*, int iAmt, sqlite3_int64 iOfst);
                   int (*xTruncate)(sqlite3_file*, sqlite3_int64 size);
                   int (*xSync)(sqlite3_file*, int flags);
                   int (*xFileSize)(sqlite3_file*, sqlite3_int64 *pSize);
                   int (*xLock)(sqlite3_file*, int);
                   int (*xUnlock)(sqlite3_file*, int);
                   int (*xCheckReservedLock)(sqlite3_file*, int *pResOut);
                   int (*xFileControl)(sqlite3_file*, int op, void *pArg);
                   int (*xSectorSize)(sqlite3_file*);
                   int (*xDeviceCharacteristics)(sqlite3_file*);
                   int (*xShmMap)(sqlite3_file*, int iPg, int pgsz, int, void volatile**);
                   int (*xShmLock)(sqlite3_file*, int offset, int n, int flags);
                   void (*xShmBarrier)(sqlite3_file*);
                   int (*xShmUnmap)(sqlite3_file*, int deleteFlag);
                   int (*xFetch)(sqlite3_file*, sqlite3_int64 iOfst, int iAmt, void **pp);
                   int (*xUnfetch)(sqlite3_file*, sqlite3_int64 iOfst, void *p);
                */
            }),
        })
    }
    pub fn ptr(&mut self) -> &mut sqlite3::sqlite3_io_methods {
        self.reference.as_mut()
    }
}

impl File {
    pub fn new() -> RefCell<Self> {
        let io_methods = Rc::new(IoMethods::new());
        RefCell::new(Self {
            io_methods: Rc::clone(&io_methods),
            reference: Box::new(sqlite3::sqlite3_file {
                pMethods: Rc::clone(&io_methods).deref().borrow_mut().ptr(),
            }),
        })
    }
}

impl Vfs {
    pub fn new() -> RefCell<Self> {
        unsafe {
            let zName = CString::new("hyper").expect("Failed to create new string for name");
            RefCell::new(Self {
                reference: Box::new(sqlite3::sqlite3_vfs {
                    iVersion: 3,
                    mxPathname: 512,
                    pNext: mem::zeroed(),
                    zName: zName.as_ptr(),
                    // I think this should hold the VFS itself. Need to refactor to allow for
                    // passing a pointer to itself.
                    pAppData: mem::zeroed(),
                    szOsFile: mem::size_of::<Box<File>>() as c_int,
                    xOpen: None,
                    xDelete: None,
                    xAccess: None,
                    xFullPathname: None,
                    xDlOpen: None,
                    xDlError: None,
                    xDlSym: None,
                    xCurrentTime: None,
                    xDlClose: None,
                    xGetLastError: None,
                    xRandomness: None,
                    xSleep: None,
                    xCurrentTimeInt64: None,
                    xGetSystemCall: None,
                    xNextSystemCall: None,
                    xSetSystemCall: None,
                    /*
                     * FIXME: Define the following functions.
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
                }),
            })
        }
    }

    pub fn register(&self) -> anyhow::Result<()> {
        let the_vfs = Vfs::new();
        let result = unsafe { sqlite3::sqlite3_vfs_register(the_vfs.borrow_mut().ptr(), 0) };

        if result == 0 {
            // We were able to register the VFS safely.
            Ok(())
        } else {
            // Something went wrong.
            panic!("What went wrong? sqlite3_vfs_register returned {}", result);
        }
    }

    fn ptr(&mut self) -> &mut sqlite3::sqlite3_vfs {
        self.reference.as_mut()
    }
}

impl Drop for Vfs {
    fn drop(&mut self) {
        let result = unsafe { sqlite3::sqlite3_vfs_unregister(self.ptr()) };
        if result != 0 {
            panic!(
                "What went wrong? sqlite3_vfs_unregister returned {}",
                result
            );
        }
    }
}

#[test]
fn test_load_vfs_successful() {
    let fs = Vfs::new();
    assert!(fs.borrow().register().is_ok());
}

#[test]
fn test_schema() {
    let schema = r#"
CREATE TABLE COMPANY (
   ID INT PRIMARY KEY     NOT NULL,
   NAME           TEXT    NOT NULL,
   AGE            INT     NOT NULL,
   ADDRESS        CHAR(50),
   SALARY         REAL
);

INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)
VALUES (1, 'Paul', 32, 'California', 20000.00 );

INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)
VALUES (2, 'Allen', 25, 'Texas', 15000.00 );

INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)
VALUES (3, 'Teddy', 23, 'Norway', 20000.00 );

INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)
VALUES (4, 'Mark', 25, 'Rich-Mond ', 65000.00 );

INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)
VALUES (5, 'David', 27, 'Texas', 85000.00 );

INSERT INTO COMPANY (ID,NAME,AGE,ADDRESS,SALARY)
VALUES (6, 'Kim', 22, 'South-Hall', 45000.00 );
    "#;
    let fs = Vfs::new();
    assert!(fs.borrow().register().is_ok());

    let conn_result: rusqlite::Result<rusqlite::Connection> = rusqlite::Connection::open_with_flags(
        "hyper://foo-network?vfs=hyper",
        rusqlite::OpenFlags::SQLITE_OPEN_URI,
    );

    assert_eq!(conn_result.as_ref().err(), None);

    let conn = conn_result.unwrap();
    conn.execute(schema, rusqlite::params![]);
}
