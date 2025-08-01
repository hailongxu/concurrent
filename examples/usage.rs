
use taskorch::{Pi, Pool, Queue, TaskBuildNew, TaskSubmitter};

/// count of a string
/// 1 ---> N
/// task-count --> (task-count1, task-count2)
/// N ---> 1
/// (task-count1, task-count2) --> task-exit


/// task flow
/// 1->N : [A]->[B1,B2,count]
/// N->1 : [B1,B2]->[Add]
/// 1->1 : [Add]->[Exit]
/// 1->1 : [count]->[Exit]
/// 
/// task belongings
/// [B1,B2,Add] in Q1
/// [A,count] in Q2
/// 
/// thread
/// Q1 with 2 thread
/// Q2 with 1 thread
/// 
fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid1 = pool.insert_queue(&Queue::new()).unwrap();
    let qid2 = pool.insert_queue(&Queue::new()).unwrap();
    let submitter1 = pool.task_submitter(qid1).unwrap();
    let submitter2 = pool.task_submitter(qid2).unwrap();

    // Step#3. create tasks
    add_task(&submitter1,&submitter2);

    // Step#4. start a thread and run
    pool.spawn_thread_for(qid1);
    pool.spawn_thread_for(qid1);
    pool.spawn_thread_for(qid2);

    // Step#5. wait until all finished
    pool.join();
}

fn add_task(submitter1:&TaskSubmitter, submitter2:&TaskSubmitter) {
    
    submitter1.submit((||println!("task='free': Hi, I'm free#11, 1 2 ..")).into_task());
    submitter1.submit((||println!("task='free': Hi, I'm free#12, 1 2 ..")).into_task());
    submitter2.submit((||println!("task='free': Hi, I'm free#2, 3 4 ..")).into_task());

    // submitter 1

    // Exit task construction
    // This elegant approach ensures all threads exit one by one,
    // guaranteeing each thread can receive the exit message
    let id_exit = submitter1.submit(
        (|_:i32| {println!("task='exit2': exit");})
        .into_exit_task())
        .unwrap();
    let id_exit = submitter1.submit(
        (|_:i32| {println!("task='exit1': exit and [1] => task='exit2'");1})
        .into_exit_task().to((id_exit,Pi::PI0).into())
    ).unwrap();

    // task add
    let id_add = submitter1.submit(
        (|a:i32,b:i32|{println!("task='add': (a:{a:?}+b:{b:?}) => task='exit'");a+b})
        .into_task().to((id_exit, Pi::PI0).into())
    ).unwrap();

    // task B1
    let id_b1 = submitter1.submit(
        (|a:i32|{println!("task='B1': recv (a:{a}) and [{a}] => task='add'");a})
        .into_task().to((id_add, Pi::PI0).into())
    ).unwrap();

    // task B2
    let id_b2 = submitter1.submit(
        (|a:i32|{println!("task='B2': recv (a:{a}) and [{a}]=> task='add'");a})
        .into_task().to((id_add, Pi::PI1).into())
    ).unwrap();

    // submitter2

    // task exit3
    let id_exit3 = submitter2.submit(
        (|_:usize| {println!("task='exit3': exit");})
        .into_exit_task()
    ).unwrap();

    // task count
    let id_count = submitter2.submit(
        (|a:&str|{println!("task='count': (a:{a:?}) and [{}] => task='exit3'",a.len());a.len()})
        .into_task().to((id_exit3, Pi::PI0).into())
    ).unwrap();

    // task A
    let _ = submitter2.submit(
        (||{println!("task='params': and pass [1,2,'123456789'] to task=['B1','B2','count']");1})
            .into_task().fan_tuple_with(
            move |_:i32| {
                (
                    (1,(id_b1,Pi::PI0).into()),
                    (2,(id_b2,Pi::PI0).into()),
                    ("123456789",(id_count,Pi::PI0).into()),
                )
            }
        )
    );
}

