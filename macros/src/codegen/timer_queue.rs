use proc_macro2::TokenStream;
use quote::quote;
use rtic_syntax::{analyze::Analysis, ast::App};

use crate::codegen::util;

const TIMER_PRIORITY: u8 = 50;

/// Generates timer queues and timer queue handlers
pub fn codegen(app: &App, _analysis: &Analysis) -> Vec<TokenStream> {
    let mut items = vec![];

    let schedule_enum = util::schedule_task_ident();

    // Enumeration of `schedule`-able tasks
    {
        let variants = app
            .software_tasks
            .iter()
            .map(|(name, task)| {
                let cfgs = &task.cfgs;

                quote!(
                    #(#cfgs)*
                    #name
                )
            })
            .collect::<Vec<_>>();

        // For future use
        // let doc = "Tasks that can be scheduled".to_string();
        items.push(quote!(
            // #[doc = #doc]
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            #[derive(Clone, Copy)]
            pub enum #schedule_enum {
                #(#variants,)*
            }
        ));
    }

    let tq_ident = util::timer_queue_ident();
    let cap: usize = app
        .software_tasks
        .iter()
        .map(|(_name, task)| task.args.capacity as usize)
        .sum();
    let cap_lit = util::capacity_literal(cap);

    items.push(quote!(
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #[allow(non_upper_case_globals)]
        rtic::export::lazy_static::lazy_static! {
            static ref #tq_ident: rtic::tq::TimerQueue<#schedule_enum, #cap_lit> = rtic::tq::TimerQueue::new().expect("Error creating timer queue");
        }
    ));

    let arms = app
        .software_tasks
        .iter()
        .map(|(name, task)| {
            let cfgs = &task.cfgs;
            let priority = task.args.priority;
            let rq = util::run_queue_ident(priority);
            let spawn_enum = util::spawn_enum_ident(priority);

            quote!(
                #(#cfgs)*
                #schedule_enum::#name => {
                    // Should never fail if capacity calculations are correct
                    #rq.0.try_send((#spawn_enum::#name, handle)).unwrap();
                }
            )
        })
        .collect::<Vec<_>>();

    let thread_ident = util::timer_queue_thread_ident();

    items.push(quote!(
        #[allow(non_snake_case)]
        fn #thread_ident() {
            /// The priority of this thread
            const PRIORITY: u8 = #TIMER_PRIORITY;

            rtic::export::set_current_thread_priority(PRIORITY).expect("Failed to set thread priority. Insufficient permissions?");

            loop {
                while let Some(nr) = #tq_ident.dequeue() {
                    let handle = nr.handle;
                    match nr.task {
                        #(#arms)*
                    }
                }

                // block here until next timer fires
                #tq_ident.wait();
            }
        }
    ));

    items
}
