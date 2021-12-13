extern crate structopt;
#[macro_use]
extern crate lazy_static;

use std::fs;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::atomic::{Ordering, AtomicI32};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use rayon::prelude::*;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Cli {
    /// The path to the file to read
    #[structopt(short = "f", long = "File", parse(from_os_str), default_value = "./data/file_100.txt")]
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

fn counta(s:&str) -> i32 {
    let mut result = 0;
    for c in s.chars() {
        if c == 'a' {
            result += 1;
        }
    }
    result
}

//TODO: add more complicated algorithm

fn report(function_to_measure: fn(&'static String) -> i32, explanation: &str) -> () {
    let start = Instant::now();
    let counter = function_to_measure(&DATA);
    let duration = start.elapsed();
    println!("{}, result is {}, time elapsed is {:?}", explanation, counter, duration);
}

fn main() {
    let options = Cli::from_args();
    println!("Running parallel tester with {} threads on {:?}", options.number_of_threads, options.path.as_path().display().to_string());
    println!("Disclaimer: note that in this evaluation order of execution matters due to cache performance");
    //let DDD: &'static str = include_str!(options.path.as_path().display().to_string());
    report(method_noparallel, "No parallel execution");
//    report(method_mutex, "Mutex with thread per line");
//    report(method_atomics, "Atomics with thread per line");
    report(method_mutex_chunks, "Mutex with chunks");
    report(method_atomics_chunks, "Atomics with chunks");
    report(method_rayon_chunks, "Rayon with chunks");
    report(method_channels_chunks, "FIXME: Channels with chunks (wrong impl)");
}

fn _chunk_size(total_size: usize) -> usize {
    match total_size {
        0..=100     => 1,
        101..=200   => 2,
        _           => 1 + total_size /(*NUMBER_OF_THREADS_REF - 1)
    }
}

fn _get_chunked_lines(contents: &'static str) -> Vec<&str> {
    contents.as_bytes()
        .chunks(_chunk_size(contents.len()))
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
}

fn method_noparallel(contents: &'static String) -> i32 {
    let lines = contents.trim().split("\n");
    lines.into_iter().map(|line| counta(line)).sum()
}

#[allow(dead_code)]
fn method_mutex(contents: &'static String) -> i32 {
    let lines = contents.trim().split("\n");
    let mut handles = vec![];
    let counter = Arc::new(Mutex::new(0i32));
    for line in lines {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let ca = counta(line);
            let mut num = counter.lock().unwrap();
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

#[allow(dead_code)]
fn method_atomics(contents: &'static String) -> i32 {
    let lines = contents.trim().split("\n");
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

fn method_mutex_chunks(contents: &'static String) -> i32 {
    let lines = _get_chunked_lines(contents);
    let mut handles = vec![];
    let counter = Arc::new(Mutex::new(0i32));
    for line in lines {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let ca = counta(line);
            let mut num = counter.lock().unwrap();
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
    };

    let mut sum :i32 = 0;
    for idx in 1..nr_of_lines {
        let r=match rx.recv() {
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