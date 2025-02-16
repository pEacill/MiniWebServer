use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool{
    // use (), because don't have value to return in handle_connection
    // threads: Vec<thread::JoinHandle<()>>,

    // use worker to instore the thread fn
    Workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        // let mut threads = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut Workers = Vec::with_capacity(size);

        for id in 0..size {
            // create threads and store them in the Vec
            Workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        // ThreadPool{threads}
        ThreadPool{
            Workers, 
            sender: Some(sender),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        // self.sender.send(job).unwrap();
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // for worker in &mut self.Workers {
        //     println!("Shytting down worker {}", worker.id);

        //     // worker.thread.join().unwrap();
        //     if let Some(thread) = worker.thread.take() {
        //         thread.join().unwrap();
        //     }
        // }

        drop(self.sender.take());

        for worker in &mut self.Workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>; 

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // let thread = thread::spawn(move || loop{
        //     let job = receiver.lock().unwrap().recv().unwrap();

        //     println!("Worker {id} got a job; executing.");

        //     job();
        // });

        let thread = thread::spawn(move || loop{
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker{
            id,
            thread: Some(thread),
        }
    }
}
