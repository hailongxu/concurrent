
use std::{
    any::Any,
    sync::atomic::{AtomicUsize, Ordering}
};

use crate::curry::{CallOnce, CallParam, Currier};

/// The kind of task.
#[derive(Clone,Copy)]
pub enum Kind {
    /// Only executing
    Normal,
    /// Exits the thread after executing
    Exit,
}

static TASK_ID:TaskIdGen = TaskIdGen::new();

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

/// gen a task ID
pub fn taskid_next()->usize {
    TASK_ID.next()
}

/// TaskBuild is defined by its intrinsic properties and may optionally include a task ID.
pub struct TaskBuild<C>(pub(crate) TaskCurrier<C>,pub(crate) Option<usize>);

/// This trait is used to convert a currier into a task build.
/// Under cond task, if id param is None, the system will automatically generate a task ID.
pub trait IntoTaskBuild where Self:Sized
{
    fn into_task(self)-> TaskBuild<Self>;
    fn into_task_to(self,to:Anchor)-> TaskBuild<Self>;
    fn into_ctask(self, id:Option<usize>) -> TaskBuild<Self>;
    fn into_ctask_to(self, id:Option<usize>, to:Anchor) -> TaskBuild<Self>;
    fn into_task_exit(self) -> TaskBuild<Self>;
    fn into_ctask_exit(self, id:Option<usize>) -> TaskBuild<Self>;
}

impl<F, C, R> IntoTaskBuild for Currier<F, C, R> {
    fn into_task(self) -> TaskBuild<Self> {
        TaskBuild(TaskCurrier {
            currier: self,
            to: None,
            kind: Kind::Normal,
        }, None)
    }
    fn into_task_to(self,to:Anchor) -> TaskBuild<Self> {
        TaskBuild(TaskCurrier {
            currier: self,
            to: Some(to),
            kind: Kind::Normal,
        }, None)
    }

    fn into_ctask(self, id:Option<usize>) -> TaskBuild<Self>
    {
        TaskBuild(TaskCurrier {
            currier: self,
            to: None,
            kind: Kind::Normal,
        }, id)
    }
    fn into_ctask_to(self, id:Option<usize>, to:Anchor) -> TaskBuild<Self>
    {
        TaskBuild(TaskCurrier {
            currier: self,
            to: Some(to),
            kind: Kind::Normal,
        }, id)
    }
    fn into_task_exit(self) -> TaskBuild<Self> {
        TaskBuild(TaskCurrier {
            currier: self,
            to: None,
            kind: Kind::Exit,
        }, None)
    }
    fn into_ctask_exit(self,id:Option<usize>) -> TaskBuild<Self> {
        TaskBuild(TaskCurrier {
            currier: self,
            to: None,
            kind: Kind::Exit,
        }, id)
    }
}

#[test]
fn test_into_task_build() {
    let f = || 42;
    let task =
        Currier::from(f)
        .into_task();
    assert_eq!(task.0.currier.call_once(), 42);
    assert!(task.1.is_none());
}

pub(crate) trait Task
{
    fn run(self:Box<Self>)->Option<Box<dyn Any>>;
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam>;
    fn kind(&self)->Kind;
}

#[derive(Clone,Copy)]
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
    // pub const fn none()->Self {
    //     Self { id: usize::MAX, i: usize::MAX }
    // }
    // pub const fn is_none(&self)->bool {
    //     self.id == usize::MAX || self.i == usize::MAX
    // }
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

/// constructs a task without cond
impl<F:FnOnce()->R,R> From<(F,Anchor,Kind)> for TaskCurrier<Currier<F,(),R>> {
    fn from((f,to,kind): (F,Anchor,Kind)) -> Self {
        Self {
            currier: Currier::from(f),
            to: Some(to),
            kind,
        }
    }
}

/// constructs a task without cond
impl<F:FnOnce()->R,R> From<(F,Anchor)> for TaskCurrier<Currier<F,(),R>> {
    fn from((f,to): (F,Anchor)) -> Self {
        Self {
            currier: Currier::from(f),
            to: Some(to),
            kind: Kind::Normal,
        }
    }
}

/// constructs a task without cond
impl<F:FnOnce()->R,R> From<(F,Kind)> for TaskCurrier<Currier<F,(),R>> {
    fn from((f,kind): (F,Kind)) -> Self {
        Self {
            currier: Currier::from(f),
            to: None,
            kind,
        }
    }
}

/// constructs a task without cond
impl<F:FnOnce()->R,R> From<F> for TaskCurrier<Currier<F,(),R>> {
    fn from(f: F) -> Self {
        Self {
            currier: Currier::from(f),
            to: None,
            kind: Kind::Normal,
        }
    }
}

/// constructs a task with 1 cond
impl<F:FnOnce(P1)->R,P1,R> From<(F,Anchor,Kind)> for TaskCurrier<Currier<F,(Option<P1>,),R>> {
    fn from((f,to,kind): (F,Anchor,Kind)) -> Self {
        Self {
            currier: Currier::from(f),
            to: Some(to),
            kind,
        }
    }
}

/// constructs a task with 1 cond
impl<F:FnOnce(P1)->R,P1,R> From<(F,Anchor)> for TaskCurrier<Currier<F,(Option<P1>,),R>> {
    fn from((f,to): (F,Anchor)) -> Self {
        Self {
            currier: Currier::from(f),
            to: Some(to),
            kind: Kind::Normal,
        }
    }
}

/// constructs a task with 1 cond
impl<F:FnOnce(P1)->R,P1,R> From<(F,Kind)> for TaskCurrier<Currier<F,(Option<P1>,),R>> {
    fn from((f,kind): (F,Kind)) -> Self {
        Self {
            currier: Currier::from(f),
            to: None,
            kind,
        }
    }
}

/// constructs a task with 1 cond
impl<F:FnOnce(P1)->R,P1,R> From<F> for TaskCurrier<Currier<F,(Option<P1>,),R>> {
    fn from(f: F) -> Self {
        Self {
            currier: Currier::from(f),
            to: None,
            kind: Kind::Normal,
        }
    }
}


/// constructs a task with 2 cond
impl<F:FnOnce(P1,P2)->R,P1,P2,R> From<(F,Anchor,Kind)> for TaskCurrier<Currier<F,(Option<P1>,Option<P2>),R>> {
    fn from((f,to,kind): (F,Anchor,Kind)) -> Self {
        Self {
            currier: Currier::from(f),
            to: Some(to),
            kind,
        }
    }
}

/// constructs a task with 2 cond
impl<F:FnOnce(P1,P2)->R,P1,P2,R> From<(F,Anchor)> for TaskCurrier<Currier<F,(Option<P1>,Option<P2>),R>> {
    fn from((f,to): (F,Anchor)) -> Self {
        Self {
            currier: Currier::from(f),
            to: Some(to),
            kind: Kind::Normal,
        }
    }
}

/// constructs a task with 2 cond
impl<F:FnOnce(P1,P2)->R,P1,P2,R> From<(F,Kind)> for TaskCurrier<Currier<F,(Option<P1>,Option<P2>),R>> {
    fn from((f,kind): (F,Kind)) -> Self {
        Self {
            currier: Currier::from(f),
            to: None,
            kind,
        }
    }
}

/// constructs a task with 2 cond
impl<F:FnOnce(P1,P2)->R,P1,P2,R> From<F> for TaskCurrier<Currier<F,(Option<P1>,Option<P2>,),R>> {
    fn from(f: F) -> Self {
        Self {
            currier: Currier::from(f),
            to: None,
            kind: Kind::Normal,
        }
    }
}



#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_from() {
        let f = ||();
        let _task 
            = TaskCurrier::from((f,Kind::Normal));
        let f = |_:i32|();
        let _task 
            = TaskCurrier::from((f,Kind::Exit));
    }
    
    #[test]
    fn test1() {
        let f = ||();
        let mut t = TaskCurrier::from((f,Kind::Exit));
        let t :Box<dyn Task> = Box::new(t);
        t.run();

        let mut t = TaskCurrier::from(f);
        let t :Box<dyn Task> = Box::new(t);
        t.run();

        let s = String::new();
        let f = ||{let s=s;};
        let fr = ||{};

        let mut t = TaskCurrier::from(f);
        let t :Box<dyn Task> = Box::new(t);
        t.run();
    }
    #[test]
    fn test_c1r1() {
        let mut v = 3;
        let f = ||{v=3;v};
        let v = Some(String::new());
        let postdo = |_:i32|{v.unwrap();};
        let _r1: &dyn FnMut()->i32 = &f;
        let r1: Box<dyn FnOnce(i32)> = Box::new(postdo);
        r1(3);

        let c1 = TaskCurrier::from((|_p:i32|(),Kind::Normal));
        let mut c1: Box<dyn Task> = Box::new(c1);
        c1.as_param_mut().map(|e|e.set(0, &5));
        c1.run();
    }
}
