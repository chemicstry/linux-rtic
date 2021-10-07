#[rtic::app]
mod app {
    use std::thread::sleep;
    use std::time::Duration;

    #[shared]
    struct Shared {
        a: u32,
        b: u32,
        c: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local, init::Monotonics) {
        task1::spawn().unwrap();
        task2::spawn().unwrap();
        task3::spawn().unwrap();

        (Shared { a: 0, b: 0, c: 0 }, Local {}, init::Monotonics())
    }

    #[task(priority = 1, shared = [a, b])]
    fn task1(mut cx: task1::Context) {
        println!("task1!");

        cx.shared.a.lock(|_a| {
            println!("task1 locked a");
            sleep(Duration::from_millis(200));

            cx.shared.b.lock(|_b| {
                println!("task1 locked b");
                sleep(Duration::from_millis(200));
            });
            println!("task1 unlocked b");
        });
        println!("task1 unlocked a");
    }

    #[task(priority = 2, shared = [b, c])]
    fn task2(mut cx: task2::Context) {
        println!("task2!");

        cx.shared.b.lock(|_b| {
            println!("task2 locked b");
            sleep(Duration::from_millis(200));

            cx.shared.c.lock(|_c| {
                println!("task2 locked c");
                sleep(Duration::from_millis(200));
            });
            println!("task2 unlocked c");
        });
        println!("task2 unlocked b");
    }

    #[task(priority = 3, shared = [c, a])]
    fn task3(mut cx: task3::Context) {
        println!("task3!");

        cx.shared.c.lock(|_c| {
            println!("task3 locked c");
            sleep(Duration::from_millis(200));

            cx.shared.a.lock(|_a| {
                println!("task3 locked a");
                sleep(Duration::from_millis(200));
            });
            println!("task3 unlocked a");
        });
        println!("task3 unlocked c");
    }
}
