# Task Orch

## System Composition
The entire concurrency library can generally be divided into four main components:

- **Task** — The minimal unit of execution, built from a fn or closure along with runtime metadata.
- **Queue** — The queue holds tasks that are waiting to be processed. It acts as a buffer where tasks are stored until they can be executed.
- **Threads** — Threads are responsible for executing the tasks retrieved from the queue.
- **Resource Pool** — Responsible for the lifecycle management of both the Queue and the Threads, establishing a mapping between names and instances.


## Task
Tasks can be orchestrated in three distinct modes — independent, dependent, and conditional — and executed concurrently to complete the full workload.

**Normal task**
|#|cond?|to-notify?|method-name|taskid-needed?|example|
|--|--|--|--|--|---|
|1|No|No|into_task|No|`Currier::from(\|\|{})->into_task()`|
|2|No|Yes|into_task_to|No|`Currier::from(\|\|{3})->into_task_to(Anchor(1,0))`|
|3|Yes|No|into_ctask|Yes|`Currier::from(\|c:i8\|{})->into_ctask(Some(1))`|
|4|Yes|Yes|into_ctask_to|Yes|`Currier::from(\|c:i8\|{9})->into_ctask_to(Some(2),Anchor(3,0))`|

**Exit Task**
|#|cond?|method-name|taskid-needed?|example|
|--|--|--|--|--|
|1|No|into_task_exit|No|`Currier::from(\|\|{})->into_task_exit()`|
|2|Yes|into_ctask_exit|Yes|`Currier::from(\|c:i8\|{})->into_ctask_exit(Some(3))`|


If the task owns cond, it must have taskid which other task can notify cond to.


## Note
As this is the initial development release (v0.1.0), the API is **highly unstable** and **will change** in subsequent versions.

## Example
```rust
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
```
