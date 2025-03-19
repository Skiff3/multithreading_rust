use std::sync::{mpsc, Arc, Mutex};
use std::time::Instant;
use std::{env, thread};

// ThreadPool struct
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(job);
        self.sender.send(job).unwrap_or_else(|err| {
            eprintln!("Failed to send job to the pool: {}", err);
        });
    }
}

// Worker struct
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv();
            match job {
                Ok(job) => {
                    let thread_id = thread::current().id();
                    println!(
                        "Worker {} (Thread ID: {:?}) is executing a job.",
                        id, thread_id
                    );
                    job();
                }
                Err(_) => {
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

// resource cleaning up
impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Shutting down...");
        for worker in self.workers.drain(..) {
            worker.thread.unwrap();
        }
    }
}

// final merge
fn merge(left: &[i32], right: &[i32]) -> Vec<i32> {
    let mut result = Vec::new();
    let (mut left_index, mut right_index) = (0, 0);

    while left_index < left.len() && right_index < right.len() {
        if left[left_index] < right[right_index] {
            result.push(left[left_index]);
            left_index += 1;
        } else {
            result.push(right[right_index]);
            right_index += 1;
        }
    }

    result.extend_from_slice(&left[left_index..]);
    result.extend_from_slice(&right[right_index..]);

    result
}

fn merge_sort(arr: &mut [i32]) {
    if arr.len() <= 1 {
        return;
    }
    let mid = arr.len() / 2;
    let mut left_half = arr[..mid].to_vec();
    let mut right_half = arr[mid..].to_vec();

    merge_sort(&mut left_half);
    merge_sort(&mut right_half);

    let sorted = merge(&left_half, &right_half);
    arr.copy_from_slice(&sorted);
}

fn par_merge_sort(arr: &mut Vec<i32>, pool: Arc<ThreadPool>, no_of_threads: usize) {
    if arr.len() <= 1 {
        return;
    }

    if no_of_threads <= 1 {
        merge_sort(arr);
        return;
    }

    let mid = arr.len() / 2;
    let left_half = arr[..mid].to_vec();
    let right_half = arr[mid..].to_vec();

    let (left_sender, left_receiver) = mpsc::channel();
    let (right_sender, right_receiver) = mpsc::channel();

    let pool_clone1 = Arc::clone(&pool);

    let pool_clone2 = Arc::clone(&pool);

    pool.execute(move || {
        let mut left = left_half;
        let thread_id = thread::current().id();
        println!("Thread {:?} is sorting left part", thread_id);
        par_merge_sort(&mut left, pool_clone1, no_of_threads / 2);
        left_sender.send(left).unwrap_or_else(|err| {
            eprintln!("Failed to send job to the pool: {}", err);
        });
    });

    pool.execute(move || {
        let mut right = right_half;
        let thread_id = thread::current().id();
        println!("Thread {:?} is sorting right part", thread_id);
        par_merge_sort(&mut right, pool_clone2, no_of_threads / 2);
        right_sender.send(right).unwrap_or_else(|err| {
            eprintln!("Failed to send job to the pool: {}", err);
        });
    });
    let left_sorted = left_receiver.recv().unwrap();
    let right_sorted = right_receiver.recv().unwrap();
    let sorted = merge(&left_sorted, &right_sorted);
    *arr = sorted;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Two arguments should be supplied");
        return;
    }
    let array_size: usize = args[1].parse().unwrap();
    let num_of_threads = args[2].parse().unwrap();
    let pool = ThreadPool::new(num_of_threads);

    if array_size <= 1_000_000 {
        println!("Err: Array size must be greater than 1,000,000.");
        return;
    }

    let mut arr: Vec<i32> = (0..array_size as i32).rev().collect(); // reversing the array

    println!(
        "Sorting an array of size {} using {} threads....",
        array_size, num_of_threads
    );

    let start_time = Instant::now();
    let pool = Arc::new(pool);

    par_merge_sort(&mut arr, Arc::clone(&pool), num_of_threads);
    let elapsed_time = start_time.elapsed();
    println!("Sorting complete in {:.2?} seconds!", elapsed_time);
}
