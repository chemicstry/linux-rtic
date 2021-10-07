// This is an example of two simultaneous priority inversions (tasks 1,2,3 and tasks 4,5,6).
// Note that the correct ordering will only be seen when running on a single core (`taskset -c 1`).
// When running on multiple cores, tasks can have arbitrary execution order, but respect resource locking.
//
// The expected (single core) completion order is as follows:
// task4 (priority ceiling to 6 due to blocking task6)
// task6
// task5 (prevented from executing before task6 by task4 ceiling)
// task1 (priority ceiling to 3 due to blocking task3)
// task3
// task2 (prevented from executing before task3 by task1 ceiling)

#[rtic::app]
mod app {
    use std::time::{Duration, Instant};

    fn busy_wait(duration: Duration) {
        let end = Instant::now() + duration;
        while Instant::now() < end {}
    }

    #[shared]
    struct Shared {
        res: u32,
        res2: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        task1::spawn_after(Duration::from_millis(0)).unwrap();
        task3::spawn_after(Duration::from_millis(100)).unwrap();
        task2::spawn_after(Duration::from_millis(200)).unwrap();
        task4::spawn_after(Duration::from_millis(300)).unwrap();
        task6::spawn_after(Duration::from_millis(400)).unwrap();
        task5::spawn_after(Duration::from_millis(500)).unwrap();

        (Shared { res: 0, res2: 0 }, Local {}, init::Monotonics())
    }

    #[task(priority = 1, shared = [res])]
    fn task1(mut cx: task1::Context) {
        println!("task1 start");

        cx.shared.res.lock(|_res| {
            busy_wait(Duration::from_millis(600));
            println!("task1 done");
        });
    }

    #[task(priority = 2)]
    fn task2(_cx: task2::Context) {
        println!("task2 start");
        busy_wait(Duration::from_millis(600));
        println!("task2 done");
    }

    #[task(priority = 3, shared = [res])]
    fn task3(mut cx: task3::Context) {
        println!("task3 start");

        cx.shared.res.lock(|_res| {
            busy_wait(Duration::from_millis(600));
            println!("task3 done");
        });
    }

    #[task(priority = 4, shared = [res2])]
    fn task4(mut cx: task4::Context) {
        println!("task4 start");

        cx.shared.res2.lock(|_res| {
            busy_wait(Duration::from_millis(600));
            println!("task4 done");
        });
    }

    #[task(priority = 5)]
    fn task5(_cx: task5::Context) {
        println!("task5 start");
        busy_wait(Duration::from_millis(600));
        println!("task5 done");
    }

    #[task(priority = 6, shared = [res2])]
    fn task6(mut cx: task6::Context) {
        println!("task6 start");

        cx.shared.res2.lock(|_res| {
            busy_wait(Duration::from_millis(600));
            println!("task6 done");
        });
    }
}
