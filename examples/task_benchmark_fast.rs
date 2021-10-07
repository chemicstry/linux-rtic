// This example benchmarks how long it takes to do 10M task switches.
// Both tasks have the same priority and run on the same thread so no kernel overhead is present.
// Completes in ~4 seconds on Raspberry Pi 4, which is 400ns per task switch.

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

    #[task]
    fn task1(_cx: task1::Context, x: i32) {
        task2::spawn(x + 1).unwrap();
    }

    #[task(local = [start])]
    fn task2(cx: task2::Context, x: i32) {
        if x < 10_000_000 {
            task1::spawn(x + 1).unwrap();
        } else {
            println!("Time: {:?}", cx.local.start.elapsed());
        }
    }
}
