use proc_macro2::TokenStream;
use quote::quote;
use rtic_syntax::{analyze::Analysis, ast::App};

mod dispatchers;
mod idle;
mod init;
mod local_resources;
mod local_resources_struct;
mod module;
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
                #(#spawn_threads)*
                #call_idle
            }
        }

        fn main() {
            unsafe { #app_name::run(); }
        }
    )
}
