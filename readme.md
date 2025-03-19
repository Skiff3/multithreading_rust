Parallel Merge Sort with Thread Pool in Rust

Overview: 
This project implements a parallel version of the Merge Sort algorithm using a thread pool in Rust. The thread pool efficiently manages worker threads, which ensures the optimal utilization of system resources while performing sorting operations in parallel.

Features
- ThreadPool implementation with configurable size.
- Parallel Merge Sort using multiple threads.
- Handles large datasets efficiently.
- Ensures the system resource clean up.

Requirements
- Rust (latest stable version)
- Command-line for execution

Code Breakdown

ThreadPool struct: 
The ThreadPool struct consists of workers and a sender which consist of a queue of jobs.

      pub struct ThreadPool {
      workers: Vec<Worker>,
      sender: mpsc::Sender<Job>,
      }

ThreadPool Methods
- new(size: usize) -> ThreadPool: Initializes the thread pool with a specified number of workers.
- execute<F>(&self, job: F): Sends a new job to the thread pool for execution of the job.

Worker struct: 
The Worker struct consists of an individual worker thread in the pool of threads.

      struct Worker {
      id: usize,
      thread: Option<thread::JoinHandle<()>>,
      }

Each worker waits for incoming jobs and executes them when received.

Parallel Merge Sort Implementation: 
The par_merge_sort function divides the array into smaller segments and sorts them in a parallel way.

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
      par_merge_sort(&mut left, pool_clone1, no_of_threads / 2);
      tx.send(left).unwrap();
      });
      
      pool.execute(move || {
      let mut right = right_half;
      par_merge_sort(&mut right, pool_clone2, no_of_threads / 2);
      tx2.send(right).unwrap();
      });
      
      let left_sorted = rx.recv().unwrap();
      let right_sorted = rx2.recv().unwrap();
      let sorted = merge(&left_sorted, &right_sorted);
      *arr = sorted;
      }


Program Execution: 
The main function initiates the thread pool and triggers the sorting process after taking the user inputs using the command line. It takes the array size and inserts the elements from 0 upto the array size in a reverse manner, so that the array can be sorted again properly.

      fn main() {
      
      let args: Vec<String> = env::args().collect();
      let array_size: usize = args[1].parse().unwrap();
      let num_of_threads = args[2].parse().unwrap();
      let pool = ThreadPool::new(num_of_threads);

      let mut arr: Vec<i32> = (0..array_size as i32).rev().collect(); // reversing the array

      let start_time = Instant::now();
      let pool = Arc::new(pool);
      par_merge_sort(&mut arr, Arc::clone(&pool), num_of_threads);
      let elapsed_time = start_time.elapsed();
      println!("Sorting complete in {:.2?} seconds!", elapsed_time);
      }


Installation & Usage

1. Build the Project

    cargo build


2. Run the Program

    cargo run <array_size> <num_of_threads>

Example:

    cargo run 10000000 8

This sorts an array of 10,000,000 elements using 8 threads.

Code Explanation
1. ThreadPool Implementation
- The `ThreadPool` struct manages a set of worker threads.
- It uses an MPSC (multi-producer, single-consumer) channel for communication and synchronization of threads.
- Each worker thread waits for jobs from the queue and executes the task.

2. Merge Sort Algorithm
- If the array size is small, it performs a normal merge sort.
- If the number of threads is greater than 1, the array is split into two halves and sorted in parallel using worker threads.
- After sorting, the two halves are merged into a final sorted array.

3. Resource Cleanup
- The Drop trait is implemented for `ThreadPool` to gracefully shut down all worker threads before exiting.
- Each worker thread stops execution when no more jobs are available.

References
- https://doc.rust-lang.org/beta/book/ch21-02-multithreaded.html
- https://doc.rust-lang.org/beta/book/ch21-03-graceful-shutdown-and-cleanup.html


Author: 
Sakib B

