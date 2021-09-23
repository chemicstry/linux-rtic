#[rtic::app]
mod app {
    use rtic::time::Duration;

    pub fn nth_prime(n: u32) -> Option<u64> {
        if n < 1 {
            return None;
        }

        // The prime counting function is pi(x) which is approximately x/ln(x)
        // A good upper bound for the nth prime is ceil(x * ln(x * ln(x)))
        let x = if n <= 10 { 10.0 } else { n as f64 };
        let limit: usize = (x * (x * (x).ln()).ln()).ceil() as usize;
        let mut sieve = vec![true; limit];
        let mut count = 0;

        // Exceptional case for 0 and 1
        sieve[0] = false;
        sieve[1] = false;

        for prime in 2..limit {
            if !sieve[prime] {
                continue;
            }
            count += 1;
            if count == n {
                return Some(prime as u64);
            }

            for multiple in ((prime * prime)..limit).step_by(prime) {
                sieve[multiple] = false;
            }
        }
        None
    }

    #[shared]
    struct Shared {
        res: u32,
    }

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        task1::spawn_after(Duration::from_millis(0)).unwrap();
        task3::spawn_after(Duration::from_millis(100)).unwrap();
        task2::spawn_after(Duration::from_millis(200)).unwrap();

        (Shared { res: 0 }, Local {}, init::Monotonics())
    }

    #[task(priority = 1, shared = [res])]
    fn task1(mut cx: task1::Context) {
        println!("task1!");

        cx.shared.res.lock(|_res| {
            println!("task1 prime 1000000: {}", nth_prime(1000000).unwrap());
        });
    }

    #[task(priority = 2)]
    fn task2(_cx: task2::Context) {
        println!("task2!");

        println!("task2 prime 1000001: {}", nth_prime(1000001).unwrap());
    }

    #[task(priority = 3, shared = [res])]
    fn task3(mut cx: task3::Context) {
        println!("task3!");

        cx.shared.res.lock(|_res| {
            println!("task3 prime 1000002: {}", nth_prime(1000002).unwrap());
        });
    }
}
