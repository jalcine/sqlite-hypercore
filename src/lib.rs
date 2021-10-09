mod hyper;
mod vfs;

use crate::vfs::VirtualFilesystem;
use rusqlite::ffi as sqlite3;
use rusqlite::Connection;
use std::cell::RefCell;
use std::ffi::CString;

pub struct HyperVirtualFilesystem {
    fs_name: CString,
    inner: Option<RefCell<sqlite3::sqlite3_vfs>>,
}

pub struct HyperVirtualFile {}

pub struct HyperSQLite {
    vfs: RefCell<HyperVirtualFilesystem>,
    feed: RefCell<hypercore::Feed<random_access_disk::RandomAccessDisk>>,
}

impl vfs::VirtualFile for HyperVirtualFile {}

impl vfs::VirtualFilesystem<HyperVirtualFile> for HyperVirtualFilesystem {
    fn get_vfs_name(&self) -> &CString {
        &self.fs_name
    }

    fn open(
        &self,
        path: &str,
        open_flags: &rusqlite::OpenFlags,
    ) -> Result<HyperVirtualFile, sqlite3::Error> {
        log::trace!(
            "We're going to try to open a file with {} at {:?}.",
            path,
            open_flags
        );
        let file = HyperVirtualFile {};
        Ok(file)
    }
    fn delete(&mut self, path: &str, sync_to_system: bool) -> Result<(), sqlite3::Error> {
        log::trace!(
            "We're going to delete a file at {} (immediately? {}).",
            path,
            sync_to_system
        );
        Ok(())
    }
    fn access(&self, path: &str, access_flags: &[vfs::AccessFlag]) -> Result<(), sqlite3::Error> {
        log::trace!(
            "We're going to check on a file for {} at {:?}",
            path,
            access_flags
        );
        Ok(())
    }

    fn full_pathname(&self, path: &str) {
        log::trace!("We're going to get the full path name of {:?}", path);
    }
}

impl AsMut<HyperVirtualFilesystem> for HyperVirtualFilesystem {
    fn as_mut(&mut self) -> &mut HyperVirtualFilesystem {
        self
    }
}

impl Drop for HyperVirtualFilesystem {
    fn drop(&mut self) {
        log::trace!("It's dropppppppppppping");
    }
}

impl HyperVirtualFilesystem {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            fs_name: CString::new("hyper-welp-okay")?,
            inner: None,
        })
    }

    pub fn register(&mut self) {
        log::trace!("Registering into SQLite's VFS...");
        // FIXME: Check if this name is already registered and return it?
        let mut fs = vfs::register_for::<HyperVirtualFile, HyperVirtualFilesystem>(self);
        log::trace!("Created the VFS struct for {:?}", self.fs_name);

        let register_result = unsafe { sqlite3::sqlite3_vfs_register(&mut fs, 0) };

        if register_result != 0 {
            panic!(
                "What went wrong? sqlite3_vfs_register returned {}",
                register_result
            );
        } else {
            self.inner = Some(RefCell::new(fs));
            log::trace!(
                "Registered the SQLite VFS for this Hypercore adapter under {:?} ({})",
                self.fs_name,
                register_result
            );
        }
    }
}

impl HyperSQLite {
    pub fn vfs_name(&self) -> String {
        self.vfs
            .borrow()
            .get_vfs_name()
            .clone()
            .into_string()
            .unwrap()
    }
    pub fn connect(&self, name: &str) -> rusqlite::Result<Connection> {
        use rusqlite::OpenFlags;

        let connection_string = format!(
            // "{}?vfs={}",
            "{}",
            name,
            // self.vfs.borrow().get_vfs_name().as_ref().to_str()?
        );
        log::trace!(
            "Attempting to open a SQLite database at {:?}",
            connection_string
        );
        // FIXME: Open a new connection using the provided name.
        Connection::open_with_flags(connection_string, OpenFlags::SQLITE_OPEN_URI)
    }

    pub async fn open(path: &str) -> anyhow::Result<Self> {
        if let Ok(feed) = hypercore::open(path).await {
            let vfs = RefCell::new(HyperVirtualFilesystem::new()?);
            vfs.borrow_mut().register();
            log::trace!(
                "Connection to Hypercore resource is active; pub-key is {}.",
                base64::encode(feed.public_key().as_bytes())
            );

            Ok(Self {
                vfs,
                feed: RefCell::new(feed),
            })
        } else {
            Err(anyhow::anyhow!("Failed to open the Hypercore system."))
        }
    }
}

#[async_std::test]
async fn hypersqlite_open() {
    let _ = env_logger::builder().is_test(true).try_init();
    let hs_result = HyperSQLite::open("hypercore-data").await;
    assert!(hs_result.is_ok());

    let hs = hs_result.unwrap();

    let conn_result = hs.connect("file:sqlite.db");
    assert_eq!(conn_result.as_ref().err(), None);

    let conn = conn_result.unwrap();
    let res = conn.execute("SELECT * from tables;", []);
    assert!(res.is_ok());
}
