use proc_macro2::TokenStream;
use rtic_syntax::{Context, analyze::Analysis, ast::App};
use quote::quote;

use crate::codegen::{module, util};

pub fn codegen(app: &App, analysis: &Analysis) -> Vec<TokenStream> {
    let mut stmts = vec![];

    for (name, task) in &app.software_tasks {
        let inputs = &task.inputs;
        let (_, _, _, input_ty) = util::regroup_inputs(inputs);

        let capacity = task.args.capacity;
        let capacity_lit = util::capacity_literal(capacity as usize);

        // Task Input Queue
        // Inputs for scheduled task are pushed into this queue
        let tiq_ident = util::task_input_queue_ident(name);
        let tiq_ty = quote!(rtic::export::TaskInputQueue<#input_ty, #capacity_lit>);
        let tiq_expr = quote!(rtic::export::TaskInputQueue::new());
        stmts.push(quote!(
            /// Queue that holds inputs for queued task
            static #tiq_ident: #tiq_ty = #tiq_expr;
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

        let shared_needs_lt = false;
        let local_needs_lt = false;

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
