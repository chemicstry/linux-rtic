use proc_macro2::TokenStream;
use quote::quote;
use rtic_syntax::{analyze::Analysis, ast::App, Context};

use crate::codegen::{local_resources_struct, module, shared_resources_struct};

/// Generates support code for `#[idle]` functions
pub fn codegen(
    app: &App,
    analysis: &Analysis,
) -> (
    // all generated idle definitions
    Vec<TokenStream>,
    // call_idle
    TokenStream,
) {
    if let Some(idle) = &app.idle {
        let mut shared_needs_lt = false;
        let mut local_needs_lt = false;
        let mut defs = vec![];

        let name = &idle.name;

        if !idle.args.shared_resources.is_empty() {
            let item = shared_resources_struct::codegen(Context::Idle, &mut shared_needs_lt, app);
            defs.push(item);
        }

        if !idle.args.local_resources.is_empty() {
            let item = local_resources_struct::codegen(Context::Idle, &mut local_needs_lt, app);
            defs.push(item);
        }

        defs.push(module::codegen(
            Context::Idle,
            shared_needs_lt,
            local_needs_lt,
            app,
            analysis,
        ));

        let attrs = &idle.attrs;
        let context = &idle.context;
        let stmts = &idle.stmts;
        defs.push(quote!(
            #(#attrs)*
            #[allow(non_snake_case)]
            fn #name(#context: #name::Context) -> ! {
                use rtic::Mutex as _;
                use rtic::mutex_prelude::*;

                #(#stmts)*
            }
        ));

        let call_idle = quote!(#name(
            #name::Context::new(&rtic::export::Priority::new(0))
        ));

        (defs, call_idle)
    } else {
        (
            vec![],
            quote!(
                let thread = std::thread::current();
                ctrlc::set_handler(move || {
                    println!("ctrl-c");
                    thread.unpark();
                }).expect("Failed to set ctrl-c handler");

                std::thread::park();
                println!("term");
            ),
        )
    }
}
