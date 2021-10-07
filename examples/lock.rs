// Note that the correct ordering will only be seen when running on a single core (`taskset -c 1`).
// When running on multiple cores, tasks can have arbitrary execution order, but respect resource locking.

#[rtic::app]
mod app {
    #[shared]
    struct Shared {
        shared: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        task1::spawn().unwrap();

        (Shared { shared: 0 }, Local {}, init::Monotonics())
    }

    // when omitted priority is assumed to be `1`
    #[task(shared = [shared])]
    fn task1(mut c: task1::Context) {
        println!("A");

        // the lower priority task requires a critical section to access the data
        c.shared.shared.lock(|shared| {
            // data can only be modified within this critical section (closure)
            *shared += 1;

            // task2 will run right away, but it will stop at `shared` lock
            task2::spawn().unwrap();

            println!("C - shared = {}", *shared);

            // task3 does not contend for `shared` so it will run to completion
            task3::spawn().unwrap();
        });

        println!("F");
    }

    #[task(priority = 2, shared = [shared])]
    fn task2(mut c: task2::Context) {
        println!("B");

        // the higher priority task does still need a critical section
        let shared = c.shared.shared.lock(|shared| {
            *shared += 1;

            *shared
        });

        println!("E - shared = {}", shared);
    }

    #[task(priority = 3)]
    fn task3(_: task3::Context) {
        println!("D");
    }
}
