use std::{
    any::Any, collections::{HashMap, VecDeque}, fmt::Debug, sync::{
        atomic::{AtomicBool, Ordering}, Arc, Condvar, Mutex
    }, thread
};

use crate::{task::{Task, Kind, Anchor}, Jhandle};
use crate::task::taskid_next;


type PostDo = dyn FnOnce(Box<dyn Any>) + Send;
// static  WHEN_NIL_COMED: Box<PostDo> = Box::new(|_|());

/// A queue holding tasks awaiting scheduling by threads
#[derive(Clone)]
pub struct Queue(Arc<(Mutex<VecDeque<(Box<dyn Task+Send>,Box<PostDo>)>>,Condvar)>);

impl Queue {
    pub fn new()->Self {
        info!("Queue created.");
        Queue(Arc::new((Mutex::new(VecDeque::new()),Condvar::new())))
    }

    pub fn add_boxtask(&self,task:Box<dyn Task+Send>, postdo: Box<PostDo>) {
        let mut lock = self.0.0.lock().unwrap();
        let is_empty = lock.is_empty();
        lock.push_back((task,postdo));
        if is_empty {
            self.0.1.notify_one();
        }
    }

    pub fn pop(&self)->Option<(Box<dyn Task+Send>,Box<PostDo>)> {
        self
            .0
            .0
            .lock()
            .unwrap()
            .pop_front()
    }
    
    pub fn clear(&self) {
        self
            .0
            .0
            .lock()
            .unwrap()
            .clear()
    }

    pub fn len(&self)->usize {
        self
            .0
            .0
            .lock()
            .unwrap()
            .len()
    }
}

pub fn spawn_thread(queue:&Queue)-> Jhandle {
    let quit_flag = Arc::<AtomicBool>::new(AtomicBool::new(false));
    let quit = quit_flag.clone();
    let queue = queue.0.clone();
    let handle = thread::spawn(move||{
        warn!("starts ok.");
        loop {
            if quit.load(Ordering::Relaxed) {
                warn!("Quit flag detected and prepare to exit.");
                break;
            }
            
            let mut m = queue.0.lock().unwrap();
            if let Some((mut task,postdo)) = m.pop_front() {
                drop(m);
                let kind = task.kind();
                let r = task.run();
                if let Some(r) = r {
                    postdo(r);
                }
                if let Kind::Exit = kind {
                    warn!("received an exit message and prepare to exit.");
                    break;
                }
            } else {
                let _unused = queue.1.wait(m);
            }
        }
        warn!("exits ok.");
    });
    Jhandle(handle,quit_flag)
}

#[derive(Clone)]
pub(crate) struct C1map(Arc<(Mutex<HashMap<usize,(Box<dyn Task+Send>,Box<PostDo>)>>,Condvar)>);

impl C1map {
    pub(crate) fn new()->Self {
        Self(
            Arc::new((Mutex::new(HashMap::new()),Condvar::new()))
        )
    }
    pub(crate) fn insert<T>(&self,task: T,postdo:Box<PostDo>,taskid:Option<usize>)->Option<usize>
    where T: Task + Send + 'static
    {
        let mut taskid = taskid;
        let taskid = *taskid.get_or_insert_with(||taskid_next());
        let task: Box::<dyn Task + Send + 'static> = Box::new(task);
        let mut lock = self.0.0.lock().unwrap();
        lock.insert(taskid, (task,postdo));
        Some(taskid)
    }
    fn remove(&self,id:usize)->Option<(Box<dyn Task+Send>,Box<PostDo>)> {
        let mut lock = self.0.0.lock().unwrap();
        lock.remove(&id)
    }

    fn update_ci<T:'static+Debug>(&self,anchor:&Anchor,v:&T)->Option<bool> {
        let mut lock = self.0.0.lock().unwrap();
        let Some((task,postdo)) = lock.get_mut(&anchor.id()) else {
            warn!("task update #{} does find", anchor.id());
            return None;
        };
        let Some(param) = task.as_param_mut() else {
            error!("task update #{} does get its params.", anchor.id());
            return None;
        };
        if !param.set(anchor.i(), v) {
            error!("task #{}.cond#{} updated by {v:?} fail.", anchor.id(), anchor.i());
            return None;
        }
        Some(param.is_full())
    }
}

pub(crate) fn when_ci_comed<T:'static+Debug>(to:&Anchor, v:T, c1map:C1map, q:Queue)->bool {
    let Some(true) = c1map.update_ci(to, &v) else {
        // the log has process in update_ci
        return false;
    };
    let Some((task,postdo)) = c1map.remove(to.id()) else {
        error!("cond task #{} does not find.",to.id());
        return  false;
    };
    q.add_boxtask(task,postdo);
    true
}

pub(crate) fn when_nil_comed() {}