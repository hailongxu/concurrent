#![doc = include_str!("../README.md")]

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread::{self, JoinHandle}
};

#[macro_use]
mod log;

mod curry;
mod queue;
mod task;
use curry::CallOnce;
use queue::{when_ci_comed, C1map};
pub use queue::{spawn_thread, Queue};
use task::Task;
pub use task::{
    Anchor,Kind,
    TaskCurrier,TaskBuildNew,TaskBuildOp,
    taskid_next,
};


/// a handle to a thread spawned for queue
pub struct Jhandle(JoinHandle<()>,Arc<AtomicBool>);

impl Jhandle {
    /// record the thread handle into pool
    pub fn collect_into(self, pool:&mut Pool)->Option<usize> {
        pool.insert_thread_handle(self)
    }

    /// the thread exit once the current task complete
    pub fn exit_next(&mut self) {
        let _ = self.1.compare_exchange(
            false, true,
            Ordering::Acquire,Ordering::Relaxed);
    }

    /// block until the thread has exited
    pub fn join(self)->thread::Result<()> {
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

    /// for Queue.id and Thread.id not Task.id
    fn next_id(&mut self)->usize {
        self.id_next += 1;
        self.id_next
    }

    pub fn task_submitter(&self, qid:usize)->Option<TaskSubmitter> {
        let queue = self.queues.get(&qid)?.clone();
        let c1map = self.c1map.clone();
        TaskSubmitter {queue, c1map}.into()
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
        // update the queue
        let _r = self.queues.insert(id, queue.clone());
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
    pub fn join(self) {
        let thcount = self.jhands.len();
        let mut exit_log = String::with_capacity(thcount*"thread(123) ".len());
        for (_innerid,handle) in self.jhands {
            let thid = handle.0.thread().id();
            if let Err(err) = handle.0.join() {
                let err = if let Some(s) = err.downcast_ref::<&str>() {
                    format!(" thread panic: {}", s)
                } else if let Some(s) = err.downcast_ref::<String>() {
                    format!("thead panic: {}", s)
                } else {
                    format!("thread panic (unknow)")
                };
                error!("{err}");
                panic!("{}", err);
            }
            let thidstr = format!("{:?} ",thid);
            exit_log.push_str(&thidstr);
            warn!("thread [{thid:?}] exits ok.");
        }
        warn!("pool with {thcount} threads: [{exit_log}] exits ok.");
    }
}


/// Handles task submission to a specific queue
#[derive(Clone)]
pub struct TaskSubmitter {
    queue: Queue,
    c1map: C1map,
}

impl TaskSubmitter {
    /// Enqueues a new task for future scheduling
    ///
    /// # argments
    /// * `task` - The task to be added, wrapped in a `TaskCurrier`.
    /// * `taskid` - An optional identifier for the task, used for tracking.
    ///
    /// # returns
    /// * `usize` - The ID of the task
    pub fn submit<C>(&self,(task,taskid):(TaskCurrier<C>,Option<usize>))->usize
        where
        TaskCurrier<C>: Task,
        C: CallOnce + Send + 'static,
        C::R: 'static + Debug,
    {
        let c1map = self.c1map.clone();
        let c1queue = self.queue.clone();
        let postdo = move |r: Box<dyn Any>| {
            let Some(to) = task.to else {
                return;
            };
            let actual_type = r.type_id();
            let Ok(r) = r.downcast::<C::R>() else {
                let _expected_type = TypeId::of::<C::R>();
                let expected_type_name = std::any::type_name::<C::R>();
                error!(
                    "to {to:?}.\ndowncast failed: expected {}, got {:?}",
                    expected_type_name, actual_type
                );
                panic!("failed to conver to R type");
                return;
            };
            let r: C::R = *r;
            when_ci_comed(&to, r, c1map.clone(), c1queue);
        };
        let postdo = Box::new(postdo);

        if 0 == task.currier.count() {
            let task = Box::new(task);
            self.queue.add_boxtask(task,postdo);
            debug!("task #{} added", usize::MAX);
            usize::MAX
        } else {
            let id = self.c1map.insert(task, postdo,taskid).unwrap();
            debug!("task with cond #{} added", id);
            id
        }
    }
}

#[test]
fn test_conv() {
    use std::any::Any;
    let a = 3i32;
    let a: &dyn Any = &a;
    let b = a.downcast_ref::<i32>();
    assert!(b.is_some());
    let b = a.downcast_ref::<i8>();
    assert!(b.is_none());
    let b = a.downcast_ref::<i64>();
    assert!(b.is_none());
}