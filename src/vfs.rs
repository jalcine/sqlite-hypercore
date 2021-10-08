use libsqlite3_sys::{sqlite3_vfs, sqlite3_vfs_register};
use std::rc::Rc;

struct Vfs {
    reference: Box<sqlite3_vfs>,
}

impl Vfs {
    pub fn new() -> Rc<Vfs> {
        Rc::new(Vfs {
            reference: Box::default(),
        })
    }

    pub fn register(&self) -> anyhow::Result<()> {
        let mut the_vfs = Vfs::new();
        let result = unsafe { sqlite3_vfs_register(the_vfs.ptr(), 0) };

        if result == 0 {
            Ok(())
        } else {
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
