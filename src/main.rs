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
            count += bytes[..bytes.len()-reminder].iter().filter(|c| **c == needle).count() as usize
        }
        count
    }
}

fn report(function_to_measure: fn(&'static String) -> i32, explanation: &str) -> () {
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

    report(method_justcounting_simd, "Just counting with SIMD");
    report(method_atomics_chunks_simd, "Atomics with SIMD");
    report(method_rayon_chunks_simd, "Rayon with SIMD");

    println!("Please be patient for the next two ...");

    report(method_atomics, "Atomics with thread per line");
    report(method_mutex, "Mutex with thread per line");
}

fn _chunk_size(total_size: usize) -> usize {
    match total_size {
        0..=100 => 1,
        101..=200 => 2,
        _ => 1 + total_size / (*NUMBER_OF_THREADS_REF - 1),
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

fn method_justcounting(contents: &'static String) -> i32 {
    counta(contents)
}

fn method_noparallel(contents: &'static String) -> i32 {
    let lines = contents.trim().split("\n");
    lines.into_iter().map(|line| counta(line)).sum()
}

fn method_justcounting_simd(contents: &'static String) -> i32 {
    occurrence_u8_simd(97 /*'a'*/, contents.as_bytes()) as i32
}

fn method_mutex(contents: &'static String) -> i32 {
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
fn method_atomics(contents: &'static String) -> i32 {
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

fn method_atomics_chunks(contents: &'static String) -> i32 {
    let lines = _get_chunked_lines(contents);
    let mut handles = vec![];
    let counter = Arc::new(AtomicI32::new(0));
    for line in lines {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let ca = counta(line);
            counter.fetch_add(ca, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    counter.load(Ordering::SeqCst)
}

fn method_atomics_chunks_simd(contents: &'static String) -> i32 {
    let lines = _get_chunked_lines(contents);
    let mut handles = vec![];
    let counter = Arc::new(AtomicI32::new(0));
    for line in lines {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let ca = occurrence_u8_simd( 97, line.as_bytes()) as i32;
            counter.fetch_add(ca, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    counter.load(Ordering::SeqCst)
}

fn method_mutex_chunks(contents: &'static String) -> i32 {
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

fn method_rayon_chunks(contents: &'static String) -> i32 {
    let lines = _get_chunked_lines(contents);
    lines.par_iter().map(|s| counta(s)).sum()
}

fn method_rayon_chunks_simd(contents: &'static String) -> i32 {
    let lines = _get_chunked_lines(contents);
    lines.par_iter().map(|s| occurrence_u8_simd(97, s.as_bytes()) as i32).sum()
}

fn method_channels_chunks(contents: &'static String) -> i32 {
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


/*
size_t memcount_avx2(const void *s, int c, size_t n)
{
  __m256i cv = _mm256_set1_epi8(c),
          zv = _mm256_setzero_si256(),
         sum = zv, acr0,acr1,acr2,acr3;
  const char *p,*pe;

  for(p = s; p != (char *)s+(n- (n % (252*32)));)
  {
    for(acr0 = acr1 = acr2 = acr3 = zv, pe = p+252*32; p != pe; p += 128)
    {
      acr0 = _mm256_sub_epi8(acr0, _mm256_cmpeq_epi8(cv, _mm256_lddqu_si256((const __m256i *)p)));
      acr1 = _mm256_sub_epi8(acr1, _mm256_cmpeq_epi8(cv, _mm256_lddqu_si256((const __m256i *)(p+32))));
      acr2 = _mm256_sub_epi8(acr2, _mm256_cmpeq_epi8(cv, _mm256_lddqu_si256((const __m256i *)(p+64))));
      acr3 = _mm256_sub_epi8(acr3, _mm256_cmpeq_epi8(cv, _mm256_lddqu_si256((const __m256i *)(p+96))));
      __builtin_prefetch(p+1024);
    }
    sum = _mm256_add_epi64(sum, _mm256_sad_epu8(acr0, zv));
    sum = _mm256_add_epi64(sum, _mm256_sad_epu8(acr1, zv));
    sum = _mm256_add_epi64(sum, _mm256_sad_epu8(acr2, zv));
    sum = _mm256_add_epi64(sum, _mm256_sad_epu8(acr3, zv));
  }

  for(acr0 = zv; p+32 < (char *)s + n; p += 32)
    acr0 = _mm256_sub_epi8(acr0, _mm256_cmpeq_epi8(cv, _mm256_lddqu_si256((const __m256i *)p)));
  sum = _mm256_add_epi64(sum, _mm256_sad_epu8(acr0, zv));

  size_t count = _mm256_extract_epi64(sum, 0)
               + _mm256_extract_epi64(sum, 1)
               + _mm256_extract_epi64(sum, 2)
               + _mm256_extract_epi64(sum, 3);

  while(p != (char *)s + n)
      count += *p++ == c;
  return count;
}
*/