use proc_macro2::TokenStream;
use quote::quote;
use rtic_syntax::{analyze::Analysis, ast::App};

use crate::codegen::util;

/// Generates task dispatchers
pub fn codegen(app: &App, analysis: &Analysis) -> Vec<TokenStream> {
    let mut stmts = vec![];

    let thread_init_barrier = util::thread_init_barrier();
    let num_threads = analysis.channels.iter().count();
    stmts.push(quote!(
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #[allow(non_upper_case_globals)]
        rtic::lazy_static::lazy_static! {
            static ref #thread_init_barrier: std::sync::Arc<std::sync::Barrier> =
               std::sync::Arc::new(std::sync::Barrier::new(#num_threads));
        }
    ));

    for (&level, channel) in &analysis.channels {
        let spawn_enum_variants = channel
            .tasks
            .iter()
            .map(|name| {
                let cfgs = &app.software_tasks[name].cfgs;

                quote!(
                    #(#cfgs)*
                    #name
                )
            })
            .collect::<Vec<_>>();

        // Enum of tasks, schedulable by this dispatcher
        let spawn_enum = util::spawn_enum_ident(level);
        stmts.push(quote!(
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy)]
            // #[doc = #doc]
            #[doc(hidden)]
            pub enum #spawn_enum {
                #(#spawn_enum_variants,)*
            }
        ));

        let capacity = channel
            .capacity
            .checked_next_power_of_two()
            .expect("task capacity too high");
        let capacity_lit = util::capacity_literal(capacity as usize);
        let rq = util::run_queue_ident(level);
        let rq_send_ty =
            quote!(rtic::mpsc::Sender<(#spawn_enum, rtic::slab::SlabHandle), #capacity_lit>);
        let rq_recv_ty = quote!(std::sync::Mutex<rtic::mpsc::Receiver<(#spawn_enum, rtic::slab::SlabHandle), #capacity_lit>>);
        let rq_expr = quote!({
            let (tx, rx) = rtic::mpsc::FutexQueue::new();
            (tx, std::sync::Mutex::new(rx))
        });

        stmts.push(quote!(
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[allow(non_upper_case_globals)]
            rtic::lazy_static::lazy_static! {
                static ref #rq: (#rq_send_ty, #rq_recv_ty) = #rq_expr;
            }
        ));

        // Generate match arms for each task
        let arms = channel
            .tasks
            .iter()
            .map(|name| {
                let task = &app.software_tasks[name];
                let cfgs = &task.cfgs;
                let input_queue = util::task_input_queue_ident(name);
                let (_, tupled, pats, _) = util::regroup_inputs(&task.inputs);
                let span_name = format!("task_{}", name);

                quote!(
                    #(#cfgs)*
                    #spawn_enum::#name => {
                        unsafe {
                            let #tupled = #input_queue.1.get_mut_unchecked().remove(handle);

                            #[cfg(feature = "profiling")]
                            let _span = rtic::tracing::span!(rtic::tracing::Level::TRACE, #span_name).entered();

                            #[cfg(feature = "profiling")]
                            rtic::tracing::trace!("running");

                            #name(
                                #name::Context::new(&core::marker::PhantomData)
                                #(,#pats)*
                            )
                        }
                    }
                )
            })
            .collect::<Vec<_>>();

        let doc = format!("Thread function to dispatch tasks at priority {}", level);
        let thread_ident = util::thread_ident(level);
        stmts.push(quote!(
            #[allow(non_snake_case)]
            #[doc = #doc]
            fn #thread_ident() {
                /// The priority of this thread
                const PRIORITY: u8 = #level;

                rtic::init_thread_state(PRIORITY);

                #[cfg(feature = "profiling")]
                rtic::tracing::trace!("thread {} waiting for init barrier", stringify!(#thread_ident));

                // Wait here until all threads have their priority set
                #thread_init_barrier.wait();

                #[cfg(feature = "profiling")]
                rtic::tracing::trace!("thread {} running", stringify!(#thread_ident));

                let mut rx = #rq.1.lock().unwrap();
                while let item = rx.recv() {
                    let (task, handle) = item.into_value();
                    match task {
                        #(#arms)*,
                    }
                }
            }
        ));
    }

    stmts
}
