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
        //println!("A");

        // the lower priority task requires a critical section to access the data
        c.shared.shared.lock(|shared| {
            // data can only be modified within this critical section (closure)
            *shared += 1;

            // task2 will *not* run right now due to the critical section
            task2::spawn().unwrap();

            //println!("B - shared = {}", *shared);

            // task3 does not contend for `shared` so it's allowed to run now
            task3::spawn().unwrap();
        });

        // critical section is over: GPIOB can now start

        //println!("E");
    }

    #[task(priority = 2, shared = [shared])]
    fn task2(mut c: task2::Context) {
        // the higher priority task does still need a critical section
        let shared = c.shared.shared.lock(|shared| {
            *shared += 1;

            *shared
        });

        //println!("D - shared = {}", shared);
    }

    #[task(priority = 3)]
    fn task3(_: task3::Context) {
        //println!("C");
    }
}
