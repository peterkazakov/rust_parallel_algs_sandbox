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
LC_CTYPE=C tr -dc "A-Za-z 0-9" < /dev/urandom | fold -w100|head -n 100 > file_hundred.txt
```

Run example with:
```
./target/release/rust_parallel_algs_sandbox -n 16
```
Example result (please note diff when native build is used wrt SIMD):
```
peterkazakov@Peters-MacBook-Pro rust_parallel_algs_sandbox % RUSTFLAGS="-C target-cpu=native" cargo run --release                                       
Running parallel tester with 16 threads on "./data/file_100.txt"
Disclaimer: note that in this evaluation order of execution matters due to cache performance
Just counting, brute force, result is 159003, time elapsed is 13.938458ms
Splitted and counted like above, result is 159003, time elapsed is 27.089388ms
Mutex with chunks, result is 159003, time elapsed is 4.821601ms
Atomics with chunks, result is 159003, time elapsed is 3.498396ms
Rayon with chunks, result is 159003, time elapsed is 2.857575ms
Channels with chunks, result is 159003, time elapsed is 2.608122ms
Just counting with SIMD, result is 159003, time elapsed is 521.032µs
Atomics with SIMD, result is 159003, time elapsed is 1.190622ms
Rayon with SIMD, result is 159003, time elapsed is 888.938µs
Please be patient for the next two ...
Atomics with thread per line, result is 159003, time elapsed is 95.896905477s
Mutex with thread per line, result is 159003, time elapsed is 98.245278407s

peterkazakov@Peters-MacBook-Pro rust_parallel_algs_sandbox %  cargo run --release 
Running parallel tester with 16 threads on "./data/file_100.txt"
Disclaimer: note that in this evaluation order of execution matters due to cache performance
Just counting, brute force, result is 159003, time elapsed is 18.096418ms
Splitted and counted like above, result is 159003, time elapsed is 38.142684ms
Mutex with chunks, result is 159003, time elapsed is 3.61126ms
Atomics with chunks, result is 159003, time elapsed is 3.734047ms
Rayon with chunks, result is 159003, time elapsed is 2.865979ms
Channels with chunks, result is 159003, time elapsed is 3.467158ms
Just counting with SIMD, result is 159003, time elapsed is 5.613338ms
Atomics with SIMD, result is 159003, time elapsed is 1.858793ms
Rayon with SIMD, result is 159003, time elapsed is 1.677242ms
Please be patient for the next two ...
Atomics with thread per line, result is 159003, time elapsed is 95.575450404s
Mutex with thread per line, result is 159003, time elapsed is 106.100522624s
```
Just counting with SIMD wins on native.

Last two executions are slow on purpose to show possible wrong parallel programming.

SIMD instructions used references:
https://medium.com/@bartekwinter3/rust-simd-a-tutorial-bb9826f98e81
https://stackoverflow.com/questions/54541129/how-to-count-character-occurrences-using-simd
