use proc_macro2::TokenStream;
use rtic_syntax::{analyze::Analysis, ast::App};
use quote::quote;

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
        let rq_send_ty = quote!(std::sync::mpsc::SyncSender<#spawn_enum>);
        // Mutex is needed because mpsc::Receiver is not Sync, but we lock it only once in processing thread
        let rq_recv_ty = quote!(std::sync::Mutex<std::sync::mpsc::Receiver<#spawn_enum>>);
        let rq_expr = quote!({
            let (send, recv) = std::sync::mpsc::sync_channel(#capacity_lit);
            (send, std::sync::Mutex::new(recv))
        });

        stmts.push(quote!(
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[allow(non_upper_case_globals)]
            rtic::export::lazy_static! {
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

                quote!(
                    #(#cfgs)*
                    #spawn_enum::#name => {
                        // Should never fail, because there is always one input item for each run queue item
                        let #tupled = #input_queue.dequeue().unwrap();

                        unsafe {
                            let priority = &rtic::export::Priority::new(PRIORITY);
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

                let rq = #rq.1.try_lock().unwrap();
                while let Ok(task) = rq.recv() {
                    match task {
                        #(#arms)*,
                    }
                }
            }
        ));
    }

    stmts
}
