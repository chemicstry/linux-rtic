use proc_macro2::TokenStream;
use quote::quote;
use rtic_syntax::{analyze::Analysis, ast::App};

mod dispatchers;
mod idle;
mod init;
mod local_resources;
mod local_resources_struct;
mod module;
mod post_init;
mod shared_resources;
mod shared_resources_struct;
mod tasks;
mod util;

pub fn app(app: &App, analysis: &Analysis) -> TokenStream {
    let app_name = &app.name;

    let user_imports = &app.user_imports;
    let user_code = &app.user_code;

    let (init_defs, call_init) = init::codegen(app, analysis);
    let (idle_defs, call_idle) = idle::codegen(app, analysis);
    let tasks = tasks::codegen(app, analysis);
    let dispatchers = dispatchers::codegen(app, analysis);
    let post_init = post_init::codegen(app, analysis);

    let mut spawn_threads = vec![];
    spawn_threads.push(quote!(
        let mut thread_handles = vec![];
    ));
    for (&level, _channel) in &analysis.channels {
        let thread_ident = util::thread_ident(level);
        let thread_name = util::thread_name(level);
        spawn_threads.push(quote!(
            let thread = std::thread::Builder::new()
                .name(#thread_name.to_string())
                .spawn(#thread_ident);

            thread_handles.push(thread);
        ));
    }

    let (mod_app_shared_resources, mod_shared_resources) = shared_resources::codegen(app, analysis);
    let (mod_app_local_resources, mod_local_resources) = local_resources::codegen(app, analysis);

    quote!(
        /// The RTIC application module
        pub mod #app_name {
            /// Unaltered user imports
            #(#user_imports)*
            /// Unaltered user code
            #(#user_code)*

            #(#tasks)*

            #(#dispatchers)*

            #(#init_defs)*
            #(#idle_defs)*

            #mod_shared_resources
            #mod_local_resources
            #(#mod_app_shared_resources)*
            #(#mod_app_local_resources)*

            #[allow(unreachable_code)]
            pub unsafe fn run() {
                #call_init
                #(#post_init)*
                #(#spawn_threads)*
                #call_idle
            }
        }

        fn main() {
            #[cfg(feature = "profiling")]
            let _guard = {
                use rtic::tracing_subscriber::prelude::*;
                let (chrome_layer, guard) = rtic::tracing_chrome::ChromeLayerBuilder::new().build();
                let fmt_layer = rtic::tracing_subscriber::fmt::layer().with_target(false);
                let filter_layer = rtic::tracing_subscriber::EnvFilter::from_default_env();
                rtic::tracing_subscriber::registry().with(chrome_layer).with(filter_layer).with(fmt_layer).init();
                guard
            };

            unsafe { #app_name::run(); }
        }
    )
}
