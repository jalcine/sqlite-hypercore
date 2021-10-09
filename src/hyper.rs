// So in this module, we need to handle the work of taking a URL that can represent a Hyper
// reference and doing work on it. This is not going to be simple. Ideally, we'd want the
// equivalent of [`co-hyperdrive`][1] to represent a folder that we can dump the SQLite bits into
// to start. However, I think it'll be safer to implement this with the equivalent Hypercore
// primitives (like locking and the like - if any).
//
// [1]: https://github.com/RangerMauve/co-hyperdrive
