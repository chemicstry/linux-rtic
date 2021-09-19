use proc_macro::TokenStream;
use rtic_syntax::Settings;
use std::{fs, path::Path};

mod codegen;

/// Attribute used to declare a RTIC application
///
/// For user documentation see the [RTIC book](https://rtic.rs)
#[proc_macro_attribute]
pub fn app(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut settings = Settings::default();
    settings.optimize_priorities = false;
    settings.parse_binds = true;
    settings.parse_extern_interrupt = true;

    let (app, analysis) = match rtic_syntax::parse(args, input, settings) {
        Err(e) => return e.to_compile_error().into(),
        Ok(x) => x,
    };

    let ts = codegen::app(&app, &analysis);

    // Try to write the expanded code to disk
    if Path::new("target").exists() {
        fs::write("target/rtic-expansion.rs", ts.to_string()).ok();
        std::process::Command::new("rustfmt")
            .arg("target/rtic-expansion.rs")
            .status()
            .ok();
    }

    ts.into()
}
