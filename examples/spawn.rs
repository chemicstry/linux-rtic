#[rtic::app]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        foo::spawn(1, 2).unwrap();

        (Shared {}, Local {}, init::Monotonics())
    }

    #[task()]
    fn foo(_c: foo::Context, x: i32, y: u32) {
        println!("foo {}, {}", x, y);
        if x < 4 {
            foo::spawn(x+1, y+2).unwrap();
        }
    }
}
