#![doc = include_str!("../README.md")]

use std::{
    any::Any,
    collections::HashMap,
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread::{self, JoinHandle}
};

mod curry;
mod task;
mod queue;
use curry::CallOnce;
use queue::{when_ci_comed, C1map};
pub use queue::{spawn_thread, Queue};
use task::Task;
use task::TaskBuild;
pub use task::TaskCurrier;
pub use task::Anchor;
pub use task::Kind;
pub use curry::Currier;
pub use task::IntoTaskBuild;


/// a handle to a thread spawned for queue
pub struct Jhandle(JoinHandle<()>,Arc<AtomicBool>);

impl Jhandle {
    /// record the thread handle into pool
    pub fn collect_into(self, pool:&mut Pool)->Option<usize> {
        pool.insert_thread_handle(self)
    }

    /// the thread exit once the current running task is finished
    pub fn exit_next(&mut self) {
        let _ = self.1.compare_exchange(
            false, true,
            Ordering::Acquire,Ordering::Relaxed);
    }

    /// block until the thread has exited
    pub fn wait(self)->thread::Result<()> {
        self.0.join()
    }
}

/// Pool, a container that holds and managers all resources, such as threads and queues
pub struct Pool {
    queues: HashMap<usize,Queue>,
    jhands: HashMap<usize,Jhandle>,
    c1map: C1map,
    id_next: usize,
}

impl Pool {
    pub fn new()-> Self {
        Self {
            queues: HashMap::new(),
            jhands: HashMap::new(),
            c1map: C1map::new(),
            id_next: 0,
        }
    }

    fn next_id(&mut self)->usize {
        self.id_next += 1;
        self.id_next
    }

    /// Enqueues a new task for future scheduling
    pub fn add<C>(&self,TaskBuild(task,taskid):TaskBuild<C>)->usize
        where
        TaskCurrier<C>: Task,
        C: CallOnce + Send + 'static,
        C::R: 'static,
    {
        let c1map = self.c1map.clone();
        let c1queue = self.queues.values().next().unwrap().clone();
        let postdo = move |r: Box<dyn Any>| {
            let (Some(to),Ok(r)) = (task.to,r.downcast::<C::R>()) else {
                assert!(false,"failed to conver to R type");
                return;
            };
            let r: C::R = *r;
            when_ci_comed(&to, r, c1map.clone(), c1queue);
        };
        let postdo = Box::new(postdo);

        if 0 == task.currier.count() {
            let task = Box::new(task);
            let (_,normal) = self.queues.iter().next().unwrap();
            normal.add_boxtask(task,postdo);
            usize::MAX
        } else {
            let id = self.c1map.insert(task, postdo,taskid).unwrap();
            id
        }
    }

    /// gets the ref to Queue by ID
    pub fn queue(&self, qid:usize)->Option<&Queue> {
        self.queues.get(&qid)
    }

    /// gets the ref to thread handle by ID
    pub fn jhandle(&self, tid:usize)->Option<&Jhandle> {
        self.jhands.get(&tid)
    }

    /// returns the queue ID recorded in pool
    pub fn insert_queue(&mut self,queue:&Queue)->Option<usize> {
        let id = self.next_id();
        let r = self.queues.insert(id, queue.clone());
        dbg!(r.is_some());
        Some(id)
    }

    /// return thread.id in pool
    pub fn spawn_thread_for(&mut self, qid:usize)->Option<usize> {
        let queue = self.queue(qid)?;
        spawn_thread(queue).collect_into(self)
    }

    fn insert_thread_handle(&mut self, jhandle:Jhandle)->Option<usize> {
        let id = self.next_id();
        self.jhands.insert(id, jhandle)
            .map(|_|id)
    }

    fn exit_next(&mut self, tid:usize)->Option<()> {
        let jhandle = self.jhands.get_mut(&tid)?;
        jhandle.exit_next();
        Some(())
    }

    /// Notifies each thread to exit upon completing each current running task.
    pub fn exit_next_all(&mut self) {
        for jhand in self.jhands.values_mut() {
            jhand.exit_next();
        }
    }

    /// block until all threads have exited
    pub fn wait(self) {
        for handle in self.jhands {
            handle.1.0.join().unwrap();
        }
    }
}
