#[rtic::app]
mod app {
    #[shared]
    struct Shared {
        shared: u32,
    }

    #[local]
    struct Local {
        times: u32
    }

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        task1::spawn().unwrap();

        (Shared { shared: 0 }, Local { times: 0 }, init::Monotonics())
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
        let _shared = c.shared.shared.lock(|shared| {
            *shared += 1;

            *shared
        });

        //println!("D - shared = {}", shared);
    }

    #[task(priority = 3)]
    fn task3(_: task3::Context) {
        //println!("C");
        task4::spawn().unwrap();
    }

    #[task(priority = 4, local = [times])]
    fn task4(c: task4::Context) {
        //println!("C");
        std::thread::sleep(std::time::Duration::from_millis(1));

        *c.local.times += 1;
        if *c.local.times < 4 {
            task1::spawn().unwrap();
        }
    }
}
