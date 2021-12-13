Sandbox to compare parallel algorithms on Rust.

Includes:
- No parallel
- Mutex
- Atomics
- Rayon
- Channels

Build in release mode:
```
RUSTFLAGS="-C target-cpu=native" cargo run --release
```

Generate random files (change params to set file size) with:
```
tr -dc "A-Za-z 0-9" < /dev/urandom | fold -w100|head -n 100 > file_hundred.txt
```

Run example with:
```
./target/release/rust_parallel_algs_sandbox -n 16
```
Example result:
```
./target/release/rust_parallel_algs_sandbox -n 16
Running parallel tester with 16 threads on "../data/file_100.txt"
Disclaimer: note that in this evaluation order of execution matters due to cache performance
No parallel execution, result is 1586477, time elapsed is 398.234677ms
Mutex with chunks, result is 1586477, time elapsed is 34.153806ms
Atomics with chunks, result is 1586477, time elapsed is 38.300712ms
Rayon with chunks, result is 1586477, time elapsed is 59.387676ms
```
