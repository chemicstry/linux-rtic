use proc_macro2::TokenStream;
use quote::quote;
use rtic_syntax::{analyze::Analysis, ast::App};

use crate::codegen::util;

/// Generates task dispatchers
pub fn codegen(app: &App, analysis: &Analysis) -> Vec<TokenStream> {
    let mut stmts = vec![];

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

        let capacity_lit = util::capacity_literal(channel.capacity as usize);
        let rq = util::run_queue_ident(level);
        let rq_send_ty = quote!(rtic::export::mpmc::Sender<(#spawn_enum, rtic::slab::SlabHandle)>);
        let rq_recv_ty =
            quote!(rtic::export::mpmc::Receiver<(#spawn_enum, rtic::slab::SlabHandle)>);
        let rq_expr = quote!(rtic::export::mpmc::bounded(#capacity_lit));

        stmts.push(quote!(
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[allow(non_upper_case_globals)]
            rtic::export::lazy_static::lazy_static! {
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
                let tracing_name = format!("task_{}", name);

                quote!(
                    #(#cfgs)*
                    #spawn_enum::#name => {
                        unsafe {
                            let #tupled = #input_queue.remove(handle);
                            let priority = &rtic::export::Priority::new(PRIORITY);
                            #[cfg(feature = "profiling")]
                            let _span = rtic::tracing::span!(rtic::tracing::Level::TRACE, #tracing_name).entered();
                            #name(
                                #name::Context::new(priority)
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

                rtic::export::set_current_thread_priority(PRIORITY).expect("Failed to set thread priority. Insufficient permissions?");

                while let Ok((task, handle)) = #rq.1.recv() {
                    match task {
                        #(#arms)*,
                    }
                }
            }
        ));
    }

    stmts
}
