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
Just counting, brute force, result is 159003, time elapsed is 10.902916ms
Splitted and counted like above, result is 159003, time elapsed is 23.699425ms
Mutex with chunks, result is 159003, time elapsed is 3.953499ms
Atomics with chunks, result is 159003, time elapsed is 2.330254ms
Rayon with chunks, result is 159003, time elapsed is 2.546256ms
Channels with chunks, result is 159003, time elapsed is 2.300136ms
Bytecount library (SIMD), suggested by Gemini, result is 159003, time elapsed is 782.881µs
Just counting with SIMD, result is 159003, time elapsed is 465.689µs
Atomics with SIMD, result is 159003, time elapsed is 1.067742ms
Rayon with SIMD, result is 159003, time elapsed is 848.601µs
Rayon with bytecount SIMD, result is 159003, time elapsed is 763.125µs
Please be patient for the next two ...
Atomics with thread per line, result is 159003, time elapsed is 95.896905477s
Mutex with thread per line, result is 159003, time elapsed is 98.245278407s

peterkazakov@Peters-MacBook-Pro rust_parallel_algs_sandbox %  cargo run --release 
Running parallel tester with 16 threads on "./data/file_100.txt"
Disclaimer: note that in this evaluation order of execution matters due to cache performance
Just counting, brute force, result is 159003, time elapsed is 10.055811ms
Splitted and counted like above, result is 159003, time elapsed is 16.786907ms
Mutex with chunks, result is 159003, time elapsed is 2.120774ms
Atomics with chunks, result is 159003, time elapsed is 2.068584ms
Rayon with chunks, result is 159003, time elapsed is 2.612559ms
Channels with chunks, result is 159003, time elapsed is 3.320887ms
Bytecount library (SIMD), suggested by Gemini, result is 159003, time elapsed is 933.423µs
Just counting with SIMD, result is 159003, time elapsed is 3.453063ms
Atomics with SIMD, result is 159003, time elapsed is 1.48469ms
Rayon with SIMD, result is 159003, time elapsed is 1.40692ms
Rayon with bytecount SIMD, result is 159003, time elapsed is 800.118µs
Please be patient for the next two ...
Atomics with thread per line, result is 159003, time elapsed is 95.575450404s
Mutex with thread per line, result is 159003, time elapsed is 106.100522624s
```
Just counting with SIMD wins on native, bytecount on generic.

Last two executions are slow on purpose to show possible wrong parallel programming.

SIMD instructions used references:
https://medium.com/@bartekwinter3/rust-simd-a-tutorial-bb9826f98e81
https://stackoverflow.com/questions/54541129/how-to-count-character-occurrences-using-simd
