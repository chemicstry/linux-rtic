// This example benchmarks how long it takes to unlock shared resource when someone is waiting on it.
// On Raspberry Pi 4 this takes 5.5us.

#[rtic::app]
mod app {
    use std::time::{Duration, Instant};

    const SAMPLES: u32 = 20_000;

    #[shared]
    struct Shared {
        a: u32,
    }

    #[local]
    #[derive(Default)]
    struct Local {
        releasing: Duration,
    }

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        task1::spawn(0).unwrap();

        (
            Shared { a: 0 },
            Local {
                releasing: Default::default(),
            },
            init::Monotonics(),
        )
    }

    #[task(priority = 2, shared = [a], local = [releasing])]
    fn task1(mut cx: task1::Context, x: u32) {
        let start = cx.shared.a.lock(|a| {
            *a += 1;

            if x < SAMPLES {
                task2::spawn(x + 1).unwrap();
            } else {
                println!("Releasing: {:?}", *cx.local.releasing / SAMPLES);
            }

            // Wait for task2 to actually block on the resource
            std::thread::sleep(Duration::from_micros(20));
            Instant::now()
        });
        *cx.local.releasing += start.elapsed();
    }

    #[task(shared = [a])]
    fn task2(mut cx: task2::Context, x: u32) {
        cx.shared.a.lock(|a| {
            *a += 1;
        });
        task1::spawn(x).unwrap();
    }
}
