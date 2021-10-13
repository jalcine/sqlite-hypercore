use super::*;
use std::ops::Deref;

struct MockFile {}

impl File for MockFile {
    fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn read(&self, amount: raw::c_int, offset: sqlite3::sqlite3_int64) -> anyhow::Result<Vec<u8>> {
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

impl System for MockFilesystem {
    fn access(&self, _path: &str, _access_flags: &[AccessFlag]) -> Result<(), sqlite3::ErrorCode> {
        todo!()
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
    ) -> Result<Box<file::WrappedFile>, sqlite3::ErrorCode> {
        log::trace!(
            "Attempting to look up the file {:?} in the mock system.",
            path
        );

        if path == "mock-system.db" {
            log::trace!("Used the expected mock file name.");
            let file_ptr = Rc::new(RefCell::new(MockFile {}));
            Ok(Box::new(file::WrappedFile::wrap(file_ptr)))
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
fn open_database_connection() {
    let _ = env_logger::builder().is_test(true).try_init();
    let mock_fs = Rc::new(RefCell::new(MockFilesystem {}));
    let inst_result = Instance::new("mock-connect", mock_fs);

    assert!(matches!(inst_result, Ok(_)));

    let inst = inst_result.unwrap();

    assert!(Instance::register(Rc::clone(&inst), false).is_ok());
    assert!(inst.deref().borrow().registered());

    log::info!("Connecting to the database...");
    let conn_result = rusqlite::Connection::open_with_flags_and_vfs(
        "mock-system.db",
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE,
        &inst.deref().borrow().vfs_name().unwrap(),
    );

    assert!(matches!(conn_result, Ok(_)));

    let conn = conn_result.unwrap();
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
}
