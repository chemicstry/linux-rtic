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
mod timer_queue;
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
        spawn_threads.push(quote!(
            thread_handles.push(std::thread::spawn(#thread_ident));
        ));
    }

    // Spawn timer thread
    let timer_thread_ident = util::timer_queue_thread_ident();
    spawn_threads.push(quote!(
        thread_handles.push(std::thread::spawn(#timer_thread_ident));
    ));

    let (mod_app_shared_resources, mod_shared_resources) = shared_resources::codegen(app, analysis);
    let (mod_app_local_resources, mod_local_resources) = local_resources::codegen(app, analysis);
    let mod_app_timer_queue = timer_queue::codegen(app, analysis);

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
            #(#mod_app_timer_queue)*

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
            use rtic::tracing_subscriber::prelude::*;
            #[cfg(feature = "profiling")]
            let (chrome_layer, guard) = rtic::tracing_chrome::ChromeLayerBuilder::new().build();
            #[cfg(feature = "profiling")]
            rtic::tracing_subscriber::registry().with(chrome_layer).init();

            unsafe { #app_name::run(); }
        }
    )
}
