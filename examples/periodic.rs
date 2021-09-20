#[rtic::app]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_cx: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn_after(std::time::Duration::from_secs(1)).unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task]
    fn foo(_cx: foo::Context) {
        println!("foo!");

        // Periodic
        foo::spawn_after(std::time::Duration::from_secs(1)).unwrap();
    }
}
