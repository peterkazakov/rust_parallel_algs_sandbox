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
Example result:
```
Running parallel tester with 16 threads on "./data/file_100.txt"
Disclaimer: note that in this evaluation order of execution matters due to cache performance
Just counting, brute force, result is 159003, time elapsed is 17.6454ms
Splitted and counted like above, result is 159003, time elapsed is 26.676ms
Mutex with chunks, result is 159003, time elapsed is 5.1727ms
Atomics with chunks, result is 159003, time elapsed is 4.7715ms
Rayon with chunks, result is 159003, time elapsed is 4.593ms
Channels with chunks, result is 159003, time elapsed is 4.5968ms
Just counting with SIMD, result is 159003, time elapsed is 2.5206ms
Atomics with SIMD (TBD:Align to match), result is 159002, time elapsed is 3.2125ms
Rayon with SIMD(TBD:Align to match), result is 159002, time elapsed is 1.7505ms

Please be patient for the next two ...
Atomics with thread per line, result is 159003, time elapsed is 850.0347629s
Mutex with thread per line, result is 159003, time elapsed is 966.8893862s
```

SIMD instructions used references:
https://medium.com/@bartekwinter3/rust-simd-a-tutorial-bb9826f98e81
https://stackoverflow.com/questions/54541129/how-to-count-character-occurrences-using-simd
