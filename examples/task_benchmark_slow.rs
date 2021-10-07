// This example benchmarks how long it takes to do 10M task switches.
// Tasks have different priorities and run on different threads so each switch involves kernel scheduler.
// Completes in ~65 seconds on Raspberry Pi 4 (single core), which is about 6.5us per task switch.

#[rtic::app]
mod app {
    use std::time::Instant;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        start: Instant,
    }

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        let start = Instant::now();

        task1::spawn(0).unwrap();

        (Shared {}, Local { start }, init::Monotonics())
    }

    #[task(priority = 1)]
    fn task1(_cx: task1::Context, x: i32) {
        task2::spawn(x + 1).unwrap();
    }

    #[task(priority = 2, local = [start])]
    fn task2(cx: task2::Context, x: i32) {
        if x < 10_000_000 {
            task1::spawn(x + 1).unwrap();
        } else {
            println!("Time: {:?}", cx.local.start.elapsed());
        }
    }
}
