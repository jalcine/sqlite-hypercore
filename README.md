# SQLite and Hypercore
[![Build Status](https://ci.jacky.wtf/api/badges/me/sqlite-hypercore/status.svg?ref=refs/heads/main)](https://ci.jacky.wtf/me/sqlite-hypercore)

> A Rust library providing [SQLite][] with an [virtual file system][vfs] to enable
> [Hypercore][] as a means of storage.

## Contributing

The primary repository for this project is stored at [git.jacky.wtf][] and mirrored to
[github.com][]. You can use whichever code forge you'd prefer.

## Things to Do

- [ ] Complete the wrapper over `sqlite3_vfs` in [`sqlite_hypercore::vfs`](./src/vfs/mod.rs).
- [ ] Implement individual database lookups by petnames into Hypercore in `sqlite_hypercore::vfs::hyper`.
- [ ] Ensure multi-thread support.
- [ ] Add tests.
- [ ] (Eventually) upstream the VFS wrapper logic to `rusqlite`.
- [ ] Figure out how to handle peering of the Hypercore backend.
- [ ] Support opening remote databases using a URL, i.e.: `hyper://$HOST/path?vfs=$HYPERCORE_VFS_NAME`
- [ ] ... and local ones i.e.: `hyper:path?vfs=$HYPERCORE_VFS_NAME` or `hyper:///full/path?vfs=$HYPERCORE_VFS_NAME`.

## End Goal

The final result is to be able to open up a connection to a Hypercore daemon 
on a local (or remote machine), find a database that can be written to and 
continue to work with SQLite as if it were a regular instance on the local machine.

```rust
use rusqlite::{Connection, OpenFlags};
use hypersqlite::{Instance, Vfs, VfsOptions, Storage};

#[async_std::main]
async fn main() -> anyhow::Result<()> {
  let mut hyper_vfs_options = VfsOptions::default();
  hyper_vfs_options.storage = Storage::InMemory;

  let hyper_vfs = Vfs::connect(hyper_vfs_options)
    .expect("Failed to connect to Hypercore daemon.");

  let inst = Instance::register("hyper-memory", hyper_vfs, false);

  let conn = Connection::open_with_flags("docs.db",
    OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
    inst.deref().borrow().vfs_name()?
    );

  // The database's been written into memory but into the Hypercore!
  Ok(())
}
```

## Licensing

This project is dual-licensed under the (BSD 2)[./LICENSE.BSD-2] and (MIT)[./LICENSE.MIT].
Just don't use it for things like (ICE)[https://www.ice.gov] or the like!

[sqlite]: https://sqlite.org/index.html
[vfs]: https://sqlite.org/vfs.html
[hypercore]: https://hypercore-protocol.org/
[github.com]: https://github.com/jalcine/sqlite-hypercore
[git.jacky.wtf]: https://git.jacky.wtf/jalcine/sqlite-hypercore
