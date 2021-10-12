// This example benchmarks how long it takes to lock/unlock shared resource without contention.
// On Raspberry Pi 4 this takes 310ns/170ns, most of it could be the Instant::now() vDSO overhead.

#[rtic::app]
mod app {
    use std::time::{Duration, Instant};

    const SAMPLES: u32 = 1_000_000;

    #[shared]
    struct Shared {
        a: u32,
    }

    #[local]
    #[derive(Default)]
    struct Local {
        taking: Duration,
        releasing: Duration,
    }

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        task1::spawn(0).unwrap();

        (
            Shared { a: 0 },
            Local {
                taking: Default::default(),
                releasing: Default::default(),
            },
            init::Monotonics(),
        )
    }

    #[task(shared = [a], local = [taking, releasing])]
    fn task1(mut cx: task1::Context, x: u32) {
        let mut start = Instant::now();
        cx.shared.a.lock(|a| {
            *cx.local.taking += start.elapsed();
            *a += 1;
            start = Instant::now();
        });
        *cx.local.releasing += start.elapsed();

        if x < SAMPLES {
            task1::spawn(x + 1).unwrap();
        } else {
            println!("Taking: {:?}", *cx.local.taking / SAMPLES);
            println!("Releasing: {:?}", *cx.local.releasing / SAMPLES);
        }
    }
}
