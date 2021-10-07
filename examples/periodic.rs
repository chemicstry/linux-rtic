#[rtic::app]
mod app {
    use std::time::{Duration, Instant};

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let deadline = Instant::now() + Duration::from_secs(1);

        // Spawn at absolute deadline
        foo::spawn_at(deadline).unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task]
    fn foo(_cx: foo::Context) {
        println!("foo!");

        // Spawn at relative deadline
        foo::spawn_after(Duration::from_secs(1)).unwrap();
    }
}
