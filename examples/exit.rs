use taskorch::{Pool, Queue, Currier, Anchor, IntoTaskBuild};

fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid = pool.insert_queue(&Queue::new()).unwrap();

    // Step#3. create tasks

    // task#1. add a indepent task
    pool.add(Currier::from(||println!("task free Fn say hello")).into_task());

    // task#2. add a exit task with cond
    let id_exit = pool.add(
        Currier::from(
            |msg:&str|println!("this is exit task, recved '{msg}' and exit")
        ).into_ctask_exit(None)
    );

    // task#3. gen a task message to notify exit task to be scheduled
    pool.add(
        Currier::from(
        move||{
            let id_exit = &id_exit;
            println!("gen a cond 'msg:exit' ref out value:{id_exit} ");
            "msg:exit"
        }).into_task_to(Anchor(id_exit, 0))
    );

    // Step#4. start a thread and run
    pool.spawn_thread_for(qid);

    // Step#5. wait until all finished
    pool.wait();
}
