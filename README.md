# SQLite and Hypercore
[![Build Status](https://ci.jacky.wtf/api/badges/me/sqlite-hypercore/status.svg?ref=refs/heads/main)](https://ci.jacky.wtf/me/sqlite-hypercore)

> A Rust library providing [SQLite][] with an [virtual file system][vfs] to enable
> [Hypercore][] as a means of storage.

## End Goal
```rust
use rusqlite::{Connection, OpenFlags};

let conn = Connection::open_with_flags_and_vfs("http://db.jacky.wtf/updates?vfs=hypercore", OpenFlags::SQLITE_OPEN_URI);

// Do your SQLite stuff.

// On another computer on a peerable network.
let conn = Connection::open_with_flags_and_vfs("http://db.jacky.wtf/updates?vfs=hypercore", OpenFlags::SQLITE_OPEN_URI);

// View data from other location with ease.
```

[sqlite]: https://sqlite.org/index.html
[vfs]: https://sqlite.org/vfs.html
[hypercore]: https://hypercore-protocol.org/
