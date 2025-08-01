
use taskorch::{Pi, Pool, Queue, TaskBuildNew, TaskSubmitter};

// Thread 1: Task execution (consumer) role
// Thread 2: Task generation (producer) role

fn main() {
    println!("----- test task orch -----");

    // Step#1. create a Pool
    let mut pool = Pool::new();

    // Step#2. create a queue
    let qid1 = pool.insert_queue(&Queue::new()).unwrap();
    let submitter1 = pool.task_submitter(qid1).unwrap();
    // Step#4. start a thread and run
    pool.spawn_thread_for(qid1);

    // Step#3. create tasks
    consume_task_prompt(&submitter1);

    std::thread::spawn(||{
        produce_task(submitter1);
    });

    // Step#5. wait until all finished
    pool.join();
}

fn consume_task_prompt(submitter:&TaskSubmitter) {
    submitter.submit((||println!("Init: waiting task to do")).into_task());
}

fn produce_task(submitter:TaskSubmitter) {
    prompt("hello");
    submitter.submit((||println!("consmue task='hello': helle everyone!")).into_task());

    prompt("exit");
    let id_exit = submitter.submit((|a:i32|println!("consume task='exit': recv cond={a} and exit.")).into_exit_task())
        .unwrap();

    prompt("add");
    let id_add = submitter.submit(
        (|a:i32,b:i32|{
            println!("consume task='add': (a={a},b={b}) and pass (r={}) to Task='exit'",a+b);
            a+b
        }).into_task().to((id_exit,Pi::PI0).into())
    ).unwrap();

    prompt("params");
    let _ = submitter.submit(
        (||{println!("consume task='params': pass (1, 2) to task='add'");1},10.into())
        .into_task()
        .fan_tuple_with(move|_:i32|
            (
                (1, (id_add, Pi::PI0).into()), // to add.cond#0
                (2, (id_add, Pi::PI1).into()), // to add.cond#1
                // (2, Anchor(id_add, 1)), /// to add.cond#1, Error, if use '///' !!!!!
            )
        ));
}

fn prompt(taskname:&'static str) {
    println!("produce task='{taskname}'.");
}
