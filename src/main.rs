extern crate structopt;
#[macro_use]
extern crate lazy_static;

use rayon::prelude::*;
use std::fs;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Cli {
    /// The path to the file to read
    #[structopt(
        short = "f",
        long = "File",
        parse(from_os_str),
        default_value = "./data/file_100.txt"
    )]
    pub path: std::path::PathBuf,
    #[structopt(short = "n", long = "Number of threads", default_value = "16")]
    pub number_of_threads: usize,
}

lazy_static! {
    static ref NUMBER_OF_THREADS_REF:usize = Cli::from_args().number_of_threads;
    static ref DATA: String = fs::read_to_string(Cli::from_args().path.as_path()).unwrap();
    // static ref DATA: &'static str = use include_str!("../data/file_1000.txt") for static file names
}

// Algorithms
#[no_mangle]
fn counta(s: &str) -> i32 {
    s.chars().filter(|c| *c == 'a').count() as i32
}

//SIMD algorithm: ref https://medium.com/@bartekwinter3/rust-simd-a-tutorial-bb9826f98e81
#[cfg(target_arch = "x86_64")]
pub fn occurrence_u8_simd(needle: u8, bytes: &[u8]) -> usize {
    use std::arch::x86_64::{
        __m256i, _mm256_cmpeq_epi8, _mm256_loadu_si256, _mm256_movemask_epi8, _mm256_set1_epi8,
    };

    unsafe {
        let needles = _mm256_set1_epi8(needle as i8);
        let mut count = 0;
        let num_iter = bytes.len() / 32;
        (0..num_iter).for_each(|i| {
            let simd_ = _mm256_loadu_si256(bytes.as_ptr().add(i * 32) as *const __m256i);
            let r = _mm256_cmpeq_epi8(needles, simd_);
            let r_moved = _mm256_movemask_epi8(r).count_ones() as usize;
            count += r_moved;
        });
        let reminder = bytes.len() % 32;
        if reminder > 0 {
            count += bytes[bytes.len() - reminder..]
                .iter()
                .filter(|c| **c == needle)
                .count()
        }
        count
    }
}

//Suggested by Gemini
#[no_mangle]
fn count_gemini(s: &str) -> i32 {
    bytecount::count(s.as_bytes(), b'a') as i32
}

fn report(function_to_measure: fn(&'static str) -> i32, explanation: &str) {
    let start = Instant::now();
    let counter = function_to_measure(&DATA);
    let duration = start.elapsed();
    println!(
        "{}, result is {}, time elapsed is {:?}",
        explanation, counter, duration
    );
}

fn main() {
    let options = Cli::from_args();
    println!(
        "Running parallel tester with {} threads on {:?}",
        options.number_of_threads,
        options.path.as_path().display().to_string()
    );
    println!("Disclaimer: note that in this evaluation order of execution matters due to cache performance");
    //let DDD: &'static str = include_str!(options.path.as_path().display().to_string());

    report(method_justcounting, "Just counting, brute force");
    report(method_noparallel, "Splitted and counted like above");

    report(method_mutex_chunks, "Mutex with chunks");
    report(method_atomics_chunks, "Atomics with chunks");
    report(method_rayon_chunks, "Rayon with chunks");
    report(method_channels_chunks, "Channels with chunks");

    report(method_bytecount, "Bytecount library (SIMD), suggested by Gemini");
    report(method_justcounting_simd, "Just counting with SIMD");
    report(method_atomics_chunks_simd, "Atomics with SIMD");
    report(method_rayon_chunks_simd, "Rayon with SIMD");
    report(method_rayon_bytecount_simd, "Rayon with bytecount SIMD");

    println!("Please be patient for the next two ...");

    report(method_atomics, "Atomics with thread per line");
    report(method_mutex, "Mutex with thread per line");
}

fn _chunk_size(total_size: usize) -> usize {
    if *NUMBER_OF_THREADS_REF <= 1 {
        total_size
    }
    else {
        match total_size {
            0..=100 => 1,
            101..=200 => 2,
            _ => 1 + total_size / (*NUMBER_OF_THREADS_REF - 1),
        }
    }
}

fn _get_chunked_lines(contents: &'static str) -> Vec<&'static str> {
    contents
        .as_bytes()
        .chunks(_chunk_size(contents.len()))
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
}

fn method_justcounting(contents: &'static str) -> i32 {
    counta(contents)
}

fn method_noparallel(contents: &'static str) -> i32 {
    let lines = contents.trim().split("\n");
    lines.into_iter().map(counta).sum()
}

fn method_justcounting_simd(contents: &'static str) -> i32 {
    occurrence_u8_simd(97 /*'a'*/, contents.as_bytes()) as i32
}

fn method_bytecount(contents: &'static str) -> i32 {
    count_gemini(contents) as i32
}

fn method_mutex(contents: &'static str) -> i32 {
    let lines = contents.trim().split("\n");
    let mut handles = vec![];
    let counter = Arc::new(Mutex::new(0i32));
    for line in lines {
        let shared_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let ca = counta(line);
            *shared_clone.lock().unwrap() += ca;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    let res = *counter.lock().unwrap();
    res
}

#[allow(dead_code)]
fn method_atomics(contents: &'static str) -> i32 {
    let lines = contents.split("\n");
    let mut handles = vec![];
    let counter = Arc::new(AtomicI32::new(0));
    for line in lines {
        let shared_counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let ca = counta(line);
            shared_counter.fetch_add(ca, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    counter.load(Ordering::SeqCst)
}

fn method_atomics_chunks(contents: &'static str) -> i32 {
    let lines = _get_chunked_lines(contents);
    let mut handles = vec![];
    let counter = Arc::new(AtomicI32::new(0));
    for line in lines {
        let int_counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let ca = counta(line);
            int_counter.fetch_add(ca, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    counter.load(Ordering::SeqCst)
}

fn method_atomics_chunks_simd(contents: &'static str) -> i32 {
    let lines = _get_chunked_lines(contents);
    let mut handles = vec![];
    let counter = Arc::new(AtomicI32::new(0));
    for line in lines {
        let int_counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let ca = occurrence_u8_simd(97, line.as_bytes()) as i32;
            int_counter.fetch_add(ca, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    counter.load(Ordering::SeqCst)
}

fn method_mutex_chunks(contents: &'static str) -> i32 {
    let lines = _get_chunked_lines(contents);
    let mut handles = vec![];
    let counter = Arc::new(Mutex::new(0i32));
    for line in lines {
        let internal_cntr = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let ca = counta(line);
            let mut num = internal_cntr.lock().unwrap();
            *num += ca;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    let res = *counter.lock().unwrap();
    res
}

fn method_rayon_chunks(contents: &'static str) -> i32 {
    let lines = _get_chunked_lines(contents);
    lines.par_iter().map(|s| counta(s)).sum()
}

fn method_rayon_chunks_simd(contents: &'static str) -> i32 {
    let lines = _get_chunked_lines(contents);
    lines
        .par_iter()
        .map(|s| occurrence_u8_simd(97, s.as_bytes()) as i32)
        .sum()
}

fn method_rayon_bytecount_simd(contents: &'static str) -> i32 {
    let lines = _get_chunked_lines(contents);
    lines
        .par_iter()
        .map(|s| count_gemini(s) as i32)
        .sum()
}

fn method_channels_chunks(contents: &'static str) -> i32 {
    let lines = _get_chunked_lines(contents);
    let (tx, rx): (Sender<i32>, Receiver<i32>) = mpsc::channel();
    let mut children = Vec::new();

    let nr_of_lines = lines.len();
    for line in lines {
        let thread_tx = tx.clone();
        let child = thread::spawn(move || {
            thread_tx.send(counta(line)).unwrap();
        });

        children.push(child);
    }

    // Wait for the threads to complete any remaining work
    for child in children {
        child.join().expect("oops! the child thread panicked");
    }

    let mut sum: i32 = 0;
    for idx in 0..nr_of_lines {
        let r = match rx.recv() {
            Ok(v) => v,
            Err(_) => {
                println!("Error collection channel {}", idx);
                0
            }
        };
        sum += r;
    }
    sum
}
