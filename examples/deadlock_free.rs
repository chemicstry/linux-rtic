#[rtic::app]
mod app {
    use std::thread::sleep;
    use std::time::Duration;

    #[shared]
    struct Shared {
        a: u32,
        b: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local, init::Monotonics) {
        task1::spawn().unwrap();
        task2::spawn().unwrap();

        (Shared { a: 0, b: 0 }, Local {}, init::Monotonics())
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

    #[task(priority = 2, shared = [a, b])]
    fn task2(mut cx: task2::Context) {
        println!("task2!");

        cx.shared.b.lock(|_b| {
            println!("task2 locked b");
            sleep(Duration::from_millis(200));

            cx.shared.a.lock(|_a| {
                println!("task2 locked a");
                sleep(Duration::from_millis(200));
            });
            println!("task2 unlocked a");
        });
        println!("task2 unlocked b");
    }
}
