use taskorch::{Pool, Queue, TaskBuildNew, TaskBuildOp};
// create 3 tasks and run 
fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid = pool.insert_queue(&Queue::new()).unwrap();
    let submitter = pool.task_submitter(qid).unwrap();

    // Step#3. create tasks

    // an indepent task
    submitter.submit((||println!("free-task:  Hello, 1 2 3 ..")).into_task());

    // an exit task with ONE str cond
    let id_exit = submitter.submit(
        (|msg:&str|
            println!("exit-task:  received ({msg:?}) and EXIT")
        ).into_exit_task()
    );

    // another task pass message to exit task
    submitter.submit(
        (||{
            const MSG: &'static str = "exit";
            println!("task 'AA':  I pass ['{MSG}'] to exit-task to exit");
            MSG
        }).into_task().to(id_exit, 0)
    );

    // Step#4. start a thread and run
    pool.spawn_thread_for(qid);

    // Step#5. wait until all finished
    pool.join();
}
