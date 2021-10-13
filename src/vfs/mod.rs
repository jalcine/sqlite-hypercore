#![allow(non_snake_case)]
use rusqlite::ffi as sqlite3;
use std::cell::RefCell;
use std::ffi::CString;
use std::mem::{self, MaybeUninit};
use std::os::raw;
use std::ptr::NonNull;
use std::rc::Rc;

mod file;
mod system;

pub use file::VirtualFile as File;
pub use system::VirtualFilesystem as System;

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

#[repr(C)]
#[derive(Clone)]
pub struct Instance {
    ptr: sqlite3::sqlite3_vfs,
    fs: Rc<RefCell<dyn System>>,
    vfs_name: CString,
}

impl Instance {
    pub fn new(
        vfs_name: impl ToString,
        filesystem: Rc<RefCell<dyn System>>,
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

    pub fn filesystem(&self) -> Rc<RefCell<dyn System>> {
        if self.fs.as_ptr().is_null() {
            panic!("Somehow, we lost the pointer.");
        } else {
            Rc::clone(&self.fs)
        }
    }

    fn into_raw(instance_rc: Rc<RefCell<Self>>) -> *mut raw::c_void {
        Box::into_raw(Box::new(Rc::clone(&instance_rc))) as *mut raw::c_void
    }

    pub fn register(instance_rc: Rc<RefCell<Self>>, make_default: bool) -> anyhow::Result<()> {
        if !instance_rc.borrow().registered() {
            {
                let mut instance_mut = instance_rc.borrow_mut();
                system::bind(&mut instance_mut.ptr);
                instance_mut.ptr.zName = instance_mut.vfs_name.as_ptr() as _;
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
                mem::forget(instance_rc);
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

    pub fn unregister(instance: &mut Self) -> anyhow::Result<()> {
        let unregister_result = unsafe { sqlite3::sqlite3_vfs_unregister(&mut instance.ptr) };

        if unregister_result == sqlite3::SQLITE_OK as _ {
            log::info!(
                "Unregistered {:?} into the SQLite VFS index.",
                instance.vfs_name
            );
            Ok(())
        } else {
            log::error!(
                "Failed to unregister {:?} into the SQLite VFS index (code: {}).",
                instance.vfs_name,
                unregister_result,
            );
            Err(anyhow::anyhow!("Failed to unregister VFS"))
        }
    }

    /// Checks to see if the `VirtualFilesystem` held by this instance has been registered.
    pub fn registered(&self) -> bool {
        NonNull::new(unsafe { sqlite3::sqlite3_vfs_find(self.vfs_name.as_ptr() as _) }).is_some()
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        if self.registered() {
            assert!(Instance::unregister(self).is_ok())
        }
        mem::drop(self.ptr);
    }
}

#[cfg(test)]
mod test;
