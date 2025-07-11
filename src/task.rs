
use std::{
    any::Any,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::curry::{CallOnce, CallParam, Currier};

/// Defines the behavior type for tasks.
#[derive(Clone,Copy)]
pub enum Kind {
    /// Standard task execution.
    /// The thread continues running after task completion.
    Normal,
    /// Exit task execution.
    /// The current thread will exit automatically after the task completes.
    Exit,
}

static TASKID:TaskIdGen = TaskIdGen::new();

struct TaskIdGen {
    nexter: AtomicUsize
}
impl TaskIdGen {
    const fn new()->Self {
        Self {
            nexter: AtomicUsize::new(0)
        }
    }
    fn next(&self)->usize {
        self.nexter.fetch_add(1, Ordering::Relaxed)
    }
}

/// Generate a task ID
pub fn taskid_next()->usize {
    TASKID.next()
}

pub(crate) trait Task
{
    fn run(self:Box<Self>)->Option<Box<dyn Any>>;
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam>;
    fn kind(&self)->Kind;
}

#[derive(Clone,Copy,Debug)]
/// Represents a position where a condition occurs.
pub struct Anchor(
    /// The task ID associated with the condition.
    pub usize,
    /// The index offset within the condition set.
    pub usize,
);

impl Anchor {
    #[inline]
    pub const fn id(&self)->usize {
        self.0
    }
    #[inline]
    pub const fn i(&self)->usize {
        self.1
    }
    #[inline]
    pub fn set(&mut self, id:usize, i:usize) {
        self.0 = id;
        self.1 = i;
    }
}

/// The carrier of the task, used to create and invoke its functionality.
pub struct TaskCurrier<Currier> {
    pub(crate) currier: Currier,
    pub(crate) to: Option<Anchor>,
    pub(crate) kind: Kind,
}

impl<T> Task for TaskCurrier<T>
    where
    T: CallOnce,
    T::R: 'static
{
    fn run(self:Box<Self>)->Option<Box<dyn Any>> {
        let r = self.currier.call_once();
        if std::mem::size_of::<T::R>() == 0 {
            None
        } else {
            Some(Box::new(r))
        }
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        self.currier.as_param_mut()
    }
    fn kind(&self)->Kind {
        self.kind
    }
}

/// A builder trait for constructing tasks with an optional task ID.
pub trait TaskBuildNew<TC> {
    /// construct a task.
    /// # Returns
    /// A tuple containing:
    /// - The `Task Currier` (`TC`)
    /// - An optional task ID (`usize`), if `None`, an ID auto-generated when needed
    fn into_task(self)->(TC,Option<usize>);

    #[deprecated(
        since="0.2.0",
        note = "Use `into_task()` instead for clearer ownership semantic. `task()` will be removed in the next release."
    )]
    fn task(self)->(TC,Option<usize>) where Self:Sized {
        self.into_task()
    }

    /// construct a exit task
    /// # Note
    /// This is functionally identical to `into_task()`, with the additional behavior of thread exit gracefully
    /// after task completion.
    fn into_exit_task(self)->(TC,Option<usize>);

    #[deprecated(
        since="0.2.0",
        note = "Use `into_exit_task()` instead for clearer ownership semantic. `exit_task()` will be removed in the next release."
    )]
    fn exit_task(self)->(TC,Option<usize>) where Self:Sized {
        self.into_exit_task()
    }
}
/// TaskBuildOp provides target anchor configuration.
pub trait TaskBuildOp<Currier> {
    /// Configures the target anchor to `(taskid, condid)`.
    /// # Arguments:
    /// * `taskid` - target task identifier
    /// * `i` - cond #index (0-based)
    fn to(self, taskid:usize,i:usize)->Self;
}

impl<Currier> TaskBuildOp<Currier> for (TaskCurrier<Currier>,Option<usize>) {
    fn to(self, taskid:usize,i:usize)->Self {
        (
            TaskCurrier {
                currier: self.0.currier,
                to: Some(Anchor(taskid,i)),
                kind: self.0.kind,
            },
            self.1
        )
    }
}

/// constructs a task without cond
impl<F:FnOnce()->R,R> TaskBuildNew<TaskCurrier<Currier<F,(),R>>> for F {
    fn into_task(self) -> (TaskCurrier<Currier<F,(),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self),
            to: None,
            kind: Kind::Normal,
        },None)
    }
    fn into_exit_task(self)->(TaskCurrier<Currier<F,(),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self),
            to: None,
            kind: Kind::Exit,
        },None)
    }
}
impl<F:FnOnce()->R,R> TaskBuildNew<TaskCurrier<Currier<F,(),R>>> for (F,usize) {
    fn into_task(self) -> (TaskCurrier<Currier<F,(),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self.0),
            to: None,
            kind: Kind::Normal,
        },Some(self.1))
    }
    fn into_exit_task(self) -> (TaskCurrier<Currier<F,(),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self.0),
            to: None,
            kind: Kind::Exit,
        },Some(self.1))
    }
}

impl<F:FnOnce(P1)->R,P1,R> TaskBuildNew<TaskCurrier<Currier<F,(P1,),R>>> for F {
    fn into_task(self) -> (TaskCurrier<Currier<F,(P1,),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self),
            to: None,
            kind: Kind::Normal,
        },None)
    }
    fn into_exit_task(self) -> (TaskCurrier<Currier<F,(P1,),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self),
            to: None,
            kind: Kind::Exit,
        },None)
    }
}
impl<F:FnOnce(P1)->R,P1,R> TaskBuildNew<TaskCurrier<Currier<F,(P1,),R>>> for (F,usize) {
    fn into_task(self) -> (TaskCurrier<Currier<F,(P1,),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self.0),
            to: None,
            kind: Kind::Normal,
        },Some(self.1))
    }
    fn into_exit_task(self) -> (TaskCurrier<Currier<F,(P1,),R>>,Option<usize>) {
        (TaskCurrier {
            currier: Currier::from(self.0),
            to: None,
            kind: Kind::Exit,
        },Some(self.1))
    }
}

macro_rules! impl_task_build_new {
    ($($P:ident),+) => {
        impl<F: FnOnce($($P),+) -> R, $($P),+, R> TaskBuildNew<TaskCurrier<Currier<F, ($($P,)+), R>>> for F {
            fn into_task(self) -> (TaskCurrier<Currier<F, ($($P,)+), R>>, Option<usize>) {
                (TaskCurrier {
                    currier: Currier::from(self),
                    to: None,
                    kind: Kind::Normal,
                }, None)
            }
            
            fn into_exit_task(self) -> (TaskCurrier<Currier<F, ($($P,)+), R>>, Option<usize>) {
                (TaskCurrier {
                    currier: Currier::from(self),
                    to: None,
                    kind: Kind::Exit,
                }, None)
            }
        }


        impl<F: FnOnce($($P),+) -> R, $($P),+, R> TaskBuildNew<TaskCurrier<Currier<F, ($($P,)+), R>>> for (F, usize) {
            fn into_task(self) -> (TaskCurrier<Currier<F, ($($P,)+), R>>, Option<usize>) {
                (TaskCurrier {
                    currier: Currier::from(self.0),
                    to: None,
                    kind: Kind::Normal,
                }, Some(self.1))
            }
            
            fn into_exit_task(self) -> (TaskCurrier<Currier<F, ($($P,)+), R>>, Option<usize>) {
                (TaskCurrier {
                    currier: Currier::from(self.0),
                    to: None,
                    kind: Kind::Exit,
                }, Some(self.1))
            }
        }
    };
}

impl_task_build_new!(P1,P2);
impl_task_build_new!(P1,P2,P3);
impl_task_build_new!(P1,P2,P3,P4);
impl_task_build_new!(P1,P2,P3,P4,P5);
impl_task_build_new!(P1,P2,P3,P4,P5,P6);
impl_task_build_new!(P1,P2,P3,P4,P5,P6,P7);
impl_task_build_new!(P1,P2,P3,P4,P5,P6,P7,P8);


#[test]
fn test_task_new() {
    let f = ||();
    let t = f.into_exit_task();
    let t :Box<dyn Task> = Box::new(t.0);
    t.run();

    let t = f.into_task();
    let t :Box<dyn Task> = Box::new(t.0);
    t.run();

    let s = String::new();
    let f = ||{let _s=s;};

    let t = f.into_task();
    let t :Box<dyn Task> = Box::new(t.0);
    t.run();
}

#[should_panic]
#[test]
fn test_task_new_panic() {
    let f = |_:i32,_:i32|{};
    let t = f.into_task();
    let t :Box<dyn Task> = Box::new(t.0);
    // the param is not set, so panic
    t.run();
}

#[test]
fn test_task_postdo() {
    let mut v = 3;
    let f = ||{v=3;v};
    let v = Some(String::new());
    let postdo = |_:i32|{v.unwrap();};
    let _r1: &dyn FnMut()->i32 = &f;
    let r1: Box<dyn FnOnce(i32)> = Box::new(postdo);
    r1(3);
}
#[test]
fn test_task_run() {
    // one cond
    let c1 = (|_p:i32|println!("get c1")).into_task();
    let mut c1: Box<dyn Task> = Box::new(c1.0);
    c1.as_param_mut().map(|e|e.set(0, &5));
    c1.run();

    // 8 cond
    let tp1 = 1;
    let tp2 = "2nd static str";
    let tp3 = "3rd String".to_string();
    let tp4 = vec![41,42,43];
    let tp5 = 5;
    let tp6 = 6;
    let tp7 = 7;
    let tp8 = 8;
    let tr8 = tp1+tp5+tp6+tp7+tp8;
    let c8 = (
        |p1:i32,
         p2:&'static str,
         p3:String,
         p4:Vec<i32>,
         p5:i32,
         p6:i32,
         p7:i32,
         p8:i32|{
        assert_eq!(p1,tp1);
        assert_eq!(p2,tp2);
        assert_eq!(p3,tp3);
        assert_eq!(p4,tp4);
        assert_eq!(p5,tp5);
        assert_eq!(p6,tp6);
        assert_eq!(p7,tp7);
        assert_eq!(p8,tp8);
        println!("recevied cond: {p1},{p2},{p3},{p4:?},{p5},{p6},{p7},{p8},");
        p1+p5+p6+p7+p8
    }).into_task();
    let mut c8: Box<dyn Task> = Box::new(c8.0);
    c8.as_param_mut().map(
        |e|
        e.set(0, &tp1) && 
        e.set(1, &tp2) && 
        e.set(2, &tp3) && 
        e.set(3, &tp4) && 
        e.set(4, &tp5) && 
        e.set(5, &tp6) && 
        e.set(6, &tp7) && 
        e.set(7, &tp8)
    );
    let r = c8.run().unwrap();
    let r = r.downcast::<i32>().unwrap();
    assert_eq!(*r, tr8);
}
