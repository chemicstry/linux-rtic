use proc_macro2::TokenStream;
use quote::quote;
use rtic_syntax::{analyze::Analysis, ast::App, Context};

use crate::codegen::{local_resources_struct, module, shared_resources_struct, util};

pub fn codegen(app: &App, analysis: &Analysis) -> Vec<TokenStream> {
    let mut stmts = vec![];

    for (name, task) in &app.software_tasks {
        let inputs = &task.inputs;
        let (_, _, _, input_ty) = util::regroup_inputs(inputs);

        let capacity = task
            .args
            .capacity
            .checked_next_power_of_two()
            .expect("task capacity too high");
        let capacity_lit = util::capacity_literal(capacity as usize);

        // Task Input Queue
        // Inputs for scheduled task are pushed into this queue
        let tiq_ident = util::task_input_queue_ident(name);
        let tiq_ty = quote!(rtic::slab::Slab<#input_ty, #capacity_lit>);
        let tiq_expr = quote!(rtic::slab::Slab::new());
        stmts.push(quote!(
            /// Queue that holds inputs for queued task
            rtic::lazy_static::lazy_static! {
                static ref #tiq_ident: #tiq_ty = #tiq_expr;
            }
        ));

        if !&task.is_extern {
            let context = &task.context;
            let attrs = &task.attrs;
            let cfgs = &task.cfgs;
            let task_stmts = &task.stmts;
            stmts.push(quote!(
                #(#attrs)*
                #(#cfgs)*
                #[allow(non_snake_case)]
                fn #name(#context: #name::Context #(,#inputs)*) {
                    use rtic::Mutex as _;
                    use rtic::mutex_prelude::*;

                    #(#task_stmts)*
                }
            ));
        }

        let mut shared_needs_lt = false;
        let mut local_needs_lt = false;

        // `${task}Locals`
        if !task.args.local_resources.is_empty() {
            let item = local_resources_struct::codegen(
                Context::SoftwareTask(name),
                &mut local_needs_lt,
                app,
            );

            stmts.push(item);
        }

        if !task.args.shared_resources.is_empty() {
            let item = shared_resources_struct::codegen(
                Context::SoftwareTask(name),
                &mut shared_needs_lt,
                app,
            );

            stmts.push(item);
        }

        // Generate task context struct and spawn function
        stmts.push(module::codegen(
            Context::SoftwareTask(name),
            shared_needs_lt,
            local_needs_lt,
            app,
            analysis,
        ));
    }

    stmts
}
