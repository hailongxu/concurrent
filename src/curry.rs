// #![feature(unboxed_closures)]

use std::{any::Any, marker::PhantomData};

// #[derive(Debug)]
pub struct Currier<F,C,R> {
    f: F,
    c: C,
    r: PhantomData<R>,
}

impl<F:FnOnce()->R,R> From<F> for Currier<F,(),R> {
    fn from(f: F) -> Self {
        Self {
            f,
            c: (),
            r: PhantomData
        }
    }
}

impl<P1,F:FnOnce(P1)->R,R> From<F> for Currier<F,(Option<P1>,),R> {
    fn from(f: F) -> Self {
        Self {
            f,
            c: (None::<P1>,),
            r: PhantomData,
        }
    }
}

impl<P1,P2,F:FnOnce(P1,P2)->R,R> From<F> for Currier<F,(Option<P1>,Option<P2>,),R> {
    fn from(f: F) -> Self {
        Self {
            f,
            c: (None::<P1>,None::<P2>),
            r: PhantomData,
        }
    }
}

#[test]
fn test2() {
    let f = ||{};
    let f1 = |_:i32|{};
    let c1 = Currier::from(f);
    let c2 = Currier::from(f1);
    println!("{:?}",c1.c);
    println!("{:?}",c2.c);
}

#[test]
fn test3<'a>() {
    struct FtOnce;
    // struct FtMut;
    // struct FtImut;
    trait Fmeta<T,R> {
        // type Fy;
        type Ftag;
        type Params;
        type R;
        // fn me(self)->Self;
    }
    impl<T:FnOnce()->R,R> Fmeta<(),R> for T {
        // type Fy = &(dyn FnOnce() + 'static);
        type Ftag = FtOnce;
        type Params = ();
        type R = R;
    }
    impl<T:FnOnce(P1)->R,P1,R> Fmeta<(P1,),R> for T {
        // type Fy = T;
        type Ftag = FtOnce;
        type Params = (P1,);
        type R = R;
    }
    fn ff<P,R>(f: impl Fmeta<P,R>)->impl Fmeta<P,R> {
        f
    }

    let a = ff(||{});
    ff(||{3});
    ff(|a:i32|a);
    ff(|a: &'a str| -> &'a str { a });
    let _: &dyn Fn() = &||{};
}


pub(crate) trait CallOnce {
    type R;
    fn call_once(self)->Self::R;
    fn count(&self)->usize;
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam>;
}

pub(crate) trait CallMut: CallOnce {
    fn call_mut(&mut self)->Self::R;
}

pub(crate) trait Call: CallMut {
    fn call(&self)->Self::R;
}

pub(crate) trait CallParam {
    fn set(&mut self, i:usize, value: &dyn Any)->bool;
    fn is_full(&self)->bool;
}

/// Fn()->R
impl<F,R> Call for &Currier<F,(),R>
where
    F: Fn()->R,
{
    fn call(&self) -> F::Output {
        (self.f)()
    }
}

impl<F,R> CallMut for &Currier<F,(),R>
where
    F: Fn()->R,
{
    fn call_mut(&mut self) -> F::Output {
        (self.f)()
    }
}

impl<F,R> CallOnce for &Currier<F,(),R>
where
    F: Fn()->R,
{
    type R = F::Output;

    fn call_once(self) -> F::Output {
        (self.f)()
    }
    fn count(&self)->usize {
        0
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        None
    }
}

/// FnMut()->R
impl<F,R> CallMut for &mut Currier<F,(),R>
where
    F: FnMut()->R,
{
    fn call_mut(&mut self) -> F::Output {
        (self.f)()
    }
}

impl<F,R> CallOnce for &mut Currier<F,(),R>
where
    F: FnMut()->R,
{
    type R = F::Output;
    fn call_once(self) -> F::Output {
        (self.f)()
    }
    fn count(&self)->usize {
        0
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        None
    }
}

// FnOnce()->R
impl<F,R> CallOnce for Currier<F,(),R>
where
    F: FnOnce()->R,
{
    type R = R;
    fn call_once(self) -> F::Output {
        (self.f)()
    }
    fn count(&self)->usize {
        0
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        None
    }
}

impl<F,R> Call for Currier<F,(),R>
where
    F: Fn()->R,
{
    fn call(&self) -> F::Output {
        (self.f)()
    }
}
impl<F,R> CallMut for Currier<F,(),R>
where
    F: FnMut()->R,
{
    fn call_mut(&mut self) -> F::Output {
        (self.f)()
    }
}


#[cfg(test)]
mod test_0 {
    use super::*;
    #[test]
    fn test1() {
        let mut c = Currier::from(||3);
        c.call();
        c.call_mut();
        c.call_once();
        let mut n = 3;
        let mut c = Currier::from(||n=4);
        c.call_mut();
        c.call_once();
        let n = String::new();
        let c = Currier::from(||{let _=n;});
        c.call_once();
    }
    fn test2() {
        trait Invoke : Fn() + FnMut() + FnOnce() {}
        impl<T> Invoke for T
        where Self: Fn() + FnMut() + FnOnce() {
        }
        let mut c = Currier::from(||3);
        c.call();
        c.call_mut();
        c.call_once();
        let mut n = 3;
        let mut c = Currier::from(||n=4);
        c.call_mut();
        c.call_once();
        let n = String::new();
        let c = Currier::from(||{let _n=n;});
        c.call_once();
    }
    #[test]
    fn test3() {
        trait InvOnce {
            fn inv_once(self);
        }
        trait InvMut: InvOnce {
            fn inv_mut(&mut self);
        }
        trait Inv: InvMut {
            fn inv(&self);
        }
        trait UniCall {
            fn go(self:Box<Self>);
        }
        struct C<F:FnOnce()>(F);
        impl <F:Fn()> Inv for C<F> {
            fn inv(&self) {
                (self.0)();
            }
        }
        impl <F:FnMut()> InvMut for C<F> {
            fn inv_mut(&mut self) {
                (self.0)();
            }
        }
        impl <F:FnOnce()> InvOnce for C<F> {
            fn inv_once(self) {
                (self.0)();
            }
        }
        impl<F:FnOnce()> UniCall for C<F> {
            fn go(self:Box<Self>) {
                self.inv_once();
            }
        }
        struct A;
        impl A {
            fn fm(&mut self) {}
        }
        let mut a = A;
        let b = &mut a;
        let f = ||{};
        let c = C(f);
        let c: Box<dyn UniCall> = Box::new(c);
        c.go();
        let fonce = ||{let _b = b;};
        let c = C(fonce);
        let c: Box<dyn UniCall> = Box::new(c);
        c.go();
        let fmut = ||{a.fm();};
        let c = C(fmut);
        let c: Box<dyn UniCall> = Box::new(c);
        c.go();
    }
}


/// Fn(P1)->R
impl<F,P1:Clone,R> Call for &Currier<F,(Option<P1>,),R>
where
    F: Fn(P1)->R,
    Self: CallParam
{
    fn call(&self) -> F::Output {
        let p1 = self.c.0.as_ref().unwrap();
        (self.f)(p1.clone())
    }
}

impl<F,P1:Clone,R> CallMut for &Currier<F,(Option<P1>,),R>
where
    F: Fn(P1)->R,
    Self: CallParam
{
    fn call_mut(&mut self) -> F::Output {
        let p1 = self.c.0.as_ref().unwrap();
        (self.f)(p1.clone())
    }
}

impl<F,P1:Clone,R> CallOnce for &Currier<F,(Option<P1>,),R>
where
    F: Fn(P1)->R,
    Self: CallParam
{
    type R = F::Output;

    fn call_once(self) -> F::Output {
        let p1 = self.c.0.as_ref().unwrap();
        (self.f)(p1.clone())
    }
    fn count(&self)->usize {
        1
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        Some(self)
    }
}

impl<F,P1:Clone,R> CallMut for &mut Currier<F,(Option<P1>,),R>
where
    F: FnMut(P1)->R,
    Self: CallParam
{
    fn call_mut(&mut self) -> F::Output {
        (self.f)(self.c.0.as_ref().unwrap().clone())
    }
}

impl<F,P1:Clone,R> CallOnce for &mut Currier<F,(Option<P1>,),R>
where
    F: FnMut(P1)->R,
    Self: CallParam
{
    type R = F::Output;
    fn call_once(self) -> F::Output {
        (self.f)(self.c.0.as_ref().unwrap().clone())
    }
    fn count(&self)->usize {
        1
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        Some(self)
    }
}


impl<F,P1,R> CallOnce for Currier<F,(Option<P1>,),R>
where
    F: FnOnce(P1)->R,
    Self: CallParam
{
    type R = R;
    fn call_once(self) -> F::Output {
        (self.f)(self.c.0.unwrap())
    }
    fn count(&self)->usize {
        1
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        Some(self)
    }
}

impl<F,P1:Clone,R> CallMut for Currier<F,(Option<P1>,),R>
where
    F: FnMut(P1)->R,
    Self: CallParam,
{
    fn call_mut(&mut self) -> F::Output {
        (self.f)(self.c.0.as_ref().unwrap().clone())
    }
}

impl<F,P1:Clone,R> Call for Currier<F,(Option<P1>,),R>
where
    F: Fn(P1)->R,
    Self: CallParam,
{
    fn call(&self) -> F::Output {
        (self.f)(self.c.0.as_ref().unwrap().clone())
    }
}


impl<F,P1:Clone+'static,R> CallParam for Currier<F,(Option<P1>,),R>
{
    fn set(&mut self, i:usize, value: &dyn Any)->bool {
        let (Some(p1),true) = (value.downcast_ref::<P1>(), i==0) else {
            return false;
        };
        self.c.0 = Some(p1.clone());
        true
    }
    fn is_full(&self)->bool {
        self.c.0.is_some()
    }
}


#[cfg(test)]
mod test_1 {
    use super::*;
    #[test]
    fn test_call() {
        let mut c = Currier::from(|a:i32|a>3);
        c.as_param_mut().unwrap().set(0, &3);
        c.call();
        c.call_mut();
        c.call_once();

        let mut c = Currier::from(|a:i32|a>3);
        c.c.0 = Some(3);
        let c = &mut c;
        c.call();
        c.call_mut();

        let mut v = 3;
        let mut c = Currier::from(|a:i32|{v=4; a>3});
        c.as_param_mut().unwrap().set(0, &3);
        c.call_mut();
        c.call_once();

        let mut c = Currier::from(|a:i32|{v=4; a>3});
        let c = &mut c;
        c.as_param_mut().unwrap().set(0, &3);
        c.call_mut();

        let v = String::new();
        let mut c = Currier::from(|a:i32|{let v=v; a>3});
        c.as_param_mut().unwrap().set(0, &3);
        c.call_once();
    }

    // the param is missing
    #[should_panic]
    fn test_panic() {
        let mut c = Currier::from(|a:i32|a>3);
        // c.as_param_mut().unwrap().set(0, &3); you must set the param first
        c.call();
        c.call_mut();
        c.call_once();
    }
}


/// Fn(P1,P2)->R
impl<F,P1:Clone,P2:Clone,R> Call for &Currier<F,(Option<P1>,Option<P2>),R>
where
    F: Fn(P1,P2)->R,
    Self: CallParam
{
    fn call(&self) -> F::Output {
        let p1 = self.c.0.as_ref().unwrap();
        let p2 = self.c.1.as_ref().unwrap();
        (self.f)(
            p1.clone(),
            p2.clone()
        )
    }
}

impl<F,P1:Clone,P2:Clone,R> CallMut for &Currier<F,(Option<P1>,Option<P2>),R>
where
    F: Fn(P1,P2)->R,
    Self: CallParam
{
    fn call_mut(&mut self) -> F::Output {
        let p1 = self.c.0.as_ref().unwrap();
        let p2 = self.c.1.as_ref().unwrap();
        (self.f)(
            p1.clone(),
            p2.clone()
        )
    }
}

impl<F,P1:Clone,P2:Clone,R> CallOnce for &Currier<F,(Option<P1>,Option<P2>,),R>
where
    F: Fn(P1,P2)->R,
    Self: CallParam
{
    type R = F::Output;

    fn call_once(self) -> F::Output {
        let p1 = self.c.0.as_ref().unwrap();
        let p2 = self.c.1.as_ref().unwrap();
        (self.f)(
            p1.clone(),
            p2.clone()
        )
    }
    fn count(&self)->usize {
        2
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        Some(self)
    }
}

impl<F,P1:Clone,P2:Clone,R> CallMut for &mut Currier<F,(Option<P1>,Option<P2>,),R>
where
    F: FnMut(P1,P2)->R,
    Self: CallParam
{
    fn call_mut(&mut self) -> F::Output {
        (self.f)(
            self.c.0.as_ref().unwrap().clone(),
            self.c.1.as_ref().unwrap().clone())
    }
}

impl<F,P1:Clone,P2:Clone,R> CallOnce for &mut Currier<F,(Option<P1>,Option<P2>,),R>
where
    F: FnMut(P1,P2)->R,
    Self: CallParam
{
    type R = F::Output;
    fn call_once(self) -> F::Output {
        (self.f)(
            self.c.0.as_ref().unwrap().clone(),
            self.c.1.as_ref().unwrap().clone(),
        )
    }
    fn count(&self)->usize {
        2
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        Some(self)
    }
}


impl<F,P1,P2,R> CallOnce for Currier<F,(Option<P1>,Option<P2>,),R>
where
    F: FnOnce(P1,P2)->R,
    Self: CallParam
{
    type R = R;
    fn call_once(self) -> F::Output {
        (self.f)(
            self.c.0.unwrap(),
            self.c.1.unwrap(),
        )
    }
    fn count(&self)->usize {
        2
    }
    fn as_param_mut(&mut self)->Option<&mut dyn CallParam> {
        Some(self)
    }
}

impl<F,P1:Clone,P2:Clone,R> CallMut for Currier<F,(Option<P1>,Option<P2>,),R>
where
    F: FnMut(P1,P2)->R,
    Self: CallParam,
{
    fn call_mut(&mut self) -> F::Output {
        (self.f)(
            self.c.0.as_ref().unwrap().clone(),
            self.c.1.as_ref().unwrap().clone(),
        )
    }
}

impl<F,P1:Clone,P2:Clone,R> Call for Currier<F,(Option<P1>,Option<P2>),R>
where
    F: Fn(P1,P2)->R,
    Self: CallParam,
{
    fn call(&self) -> F::Output {
        (self.f)(
            self.c.0.as_ref().unwrap().clone(),
            self.c.1.as_ref().unwrap().clone(),
        )
    }
}


impl<F,P1:Clone+'static,P2:Clone+'static,R> CallParam for Currier<F,(Option<P1>,Option<P2>),R>
{
    fn set(&mut self, i:usize, value: &dyn Any)->bool {
        match i {
            0 => {
                let Some(p1) = value.downcast_ref::<P1>() else {
                    return false;
                };
                self.c.0 = Some(p1.clone());
                true
            }
            1 => {
                let Some(p2) = value.downcast_ref::<P2>() else {
                    return false;
                };
                self.c.1 = Some(p2.clone());
                true
            }
            _ => false
        }
    }
    fn is_full(&self)->bool {
        self.c.0.is_some() &&
        self.c.1.is_some()
    }
}


#[cfg(test)]
mod test_2 {
    use super::*;
    #[test]
    fn test_call() {
        let mut c = Currier::from(|a:i32,b:i32|a<b);
        c.as_param_mut().unwrap().set(0, &3);
        c.as_param_mut().unwrap().set(1, &4);
        c.call();
        c.call_mut();
        c.call_once();

        let mut c = Currier::from(|a:i32,b:i32|a<b);
        let c = &mut c;
        c.as_param_mut().unwrap().set(0, &3);
        c.as_param_mut().unwrap().set(1, &4);
        c.call();
        c.call_mut();

        let mut v = 3;
        let mut c = Currier::from(|a:i32,b:i32|{v=4; a<b});
        c.as_param_mut().unwrap().set(0, &3);
        c.as_param_mut().unwrap().set(1, &4);
        c.call_mut();
        c.call_once();

        let mut c = Currier::from(|a:i32,b:i32|{v=4; a<b});
        let c = &mut c;
        c.as_param_mut().unwrap().set(0, &3);
        c.as_param_mut().unwrap().set(1, &4);
        c.call_mut();

        let v = String::new();
        let mut c = Currier::from(|a:i32,b:i32|{let v=v; a<b});
        c.as_param_mut().unwrap().set(0, &3);
        c.as_param_mut().unwrap().set(1, &4);
        c.call_once();
    }
}

