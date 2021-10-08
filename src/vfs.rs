use libsqlite3_sys::{sqlite3_vfs, sqlite3_vfs_register};
use std::convert::TryInto;
use std::ffi::CString;
use std::mem;
use std::rc::Rc;

struct Vfs {
    reference: Box<sqlite3_vfs>,
}

struct File {}

/// This _should_ be where all of the logic for working with Hypercore and SQLite happen.
impl Vfs {
    pub fn new() -> Rc<Vfs> {
        unsafe {
            let zName = CString::new("hyper").expect("Failed to create new string for name");
            Rc::new(Vfs {
                reference: Box::new(sqlite3_vfs {
                    iVersion: 3,
                    mxPathname: 512,
                    pNext: mem::zeroed(),
                    zName: zName.as_ptr(),
                    pAppData: mem::zeroed(),
                    szOsFile: mem::size_of::<Box<File>>()
                        .try_into()
                        .expect("Could not get the size of a file"),
                }),
            })
        }
    }

    pub fn register(&self) -> anyhow::Result<()> {
        let mut the_vfs = Vfs::new();
        let result = unsafe { sqlite3_vfs_register(the_vfs.ptr(), 0) };

        if result == 0 {
            // We were able to register the VFS safely.
            Ok(())
        } else {
            // Something went wrong.
            panic!("What went wrong? sqlite3_vfs_register returned {}", result);
        }
    }

    fn ptr(&self) -> &mut sqlite3_vfs {
        self.reference.as_mut()
    }
}

#[test]
fn test_load_vfs_successful() {
    let fs = Vfs::new();
    assert!(fs.register().is_ok());
}
