use proc_macro2::{Span, TokenStream};
use quote::quote;
use rtic_syntax::{ast::App, Context};
use syn::{Ident, LitInt, PatType};

const RTIC_INTERNAL: &str = "__rtic_internal";

/// Mark a name as internal
pub fn mark_internal_name(name: &str) -> Ident {
    Ident::new(&format!("{}_{}", RTIC_INTERNAL, name), Span::call_site())
}

/// Identifier for the task input queue
pub fn task_input_queue_ident(task: &Ident) -> Ident {
    mark_internal_name(&format!("{}_input_queue", task.to_string()))
}

/// Identifier for the run queue
pub fn run_queue_ident(priority: u8) -> Ident {
    mark_internal_name(&format!("P{}_run_queue", priority))
}

/// Identifier for the barrier which wait for all threads to be initialized
pub fn thread_init_barrier() -> Ident {
    mark_internal_name(&format!("thread_init_barrier"))
}

/// Generates an identifier for a thread that executes tasks at a given priority level
pub fn thread_ident(priority: u8) -> Ident {
    mark_internal_name(&format!("thread_P{}", priority))
}

/// Generates an OS thread name
pub fn thread_name(priority: u8) -> String {
    format!("thd_P{}", priority)
}

/// Generates an identifier for the `enum` of `spawn`-able tasks
///
/// This identifier needs the same structure as the `RQ` identifier because there's one ready queue
/// for each of these `T` enums
pub fn spawn_enum_ident(priority: u8) -> Ident {
    Ident::new(&format!("P{}_tasks", priority), Span::call_site())
}

/// Generate an internal identifier for task context
pub fn internal_task_context_ident(task: &Ident) -> Ident {
    mark_internal_name(&format!("{}_context", task.to_string()))
}

/// Generate an internal identifier for task spawn function
pub fn internal_task_spawn_ident(task: &Ident) -> Ident {
    mark_internal_name(&format!("{}_spawn", task.to_string()))
}

/// Generate an internal identifier for task spawn at function
pub fn internal_task_spawn_at_ident(task: &Ident) -> Ident {
    mark_internal_name(&format!("{}_spawn_at", task.to_string()))
}

/// Generate an internal identifier for task spawn after function
pub fn internal_task_spawn_after_ident(task: &Ident) -> Ident {
    mark_internal_name(&format!("{}_spawn_after", task.to_string()))
}

/// Generates a pre-reexport identifier for the "shared resources" struct
pub fn shared_resources_ident(ctxt: Context, app: &App) -> Ident {
    let mut s = match ctxt {
        Context::Init => app.init.name.to_string(),
        Context::Idle => app.idle.as_ref().unwrap().name.to_string(),
        Context::HardwareTask(ident) | Context::SoftwareTask(ident) => ident.to_string(),
    };

    s.push_str("SharedResources");

    mark_internal_name(&s)
}

/// Generates a pre-reexport identifier for the "local resources" struct
pub fn local_resources_ident(ctxt: Context, app: &App) -> Ident {
    let mut s = match ctxt {
        Context::Init => app.init.name.to_string(),
        Context::Idle => app.idle.as_ref().unwrap().name.to_string(),
        Context::HardwareTask(ident) | Context::SoftwareTask(ident) => ident.to_string(),
    };

    s.push_str("LocalResources");

    mark_internal_name(&s)
}

/// Turns `capacity` into an unsuffixed integer literal
pub fn capacity_literal(capacity: usize) -> LitInt {
    LitInt::new(&capacity.to_string(), Span::call_site())
}

/// Regroups the inputs of a task
///
/// `inputs` could be &[`input: Foo`] OR &[`mut x: i32`, `ref y: i64`]
pub fn regroup_inputs(
    inputs: &[PatType],
) -> (
    // args e.g. &[`_0`],  &[`_0: i32`, `_1: i64`]
    Vec<TokenStream>,
    // tupled e.g. `_0`, `(_0, _1)`
    TokenStream,
    // untupled e.g. &[`_0`], &[`_0`, `_1`]
    Vec<TokenStream>,
    // ty e.g. `Foo`, `(i32, i64)`
    TokenStream,
) {
    if inputs.len() == 1 {
        let ty = &inputs[0].ty;

        (
            vec![quote!(_0: #ty)],
            quote!(_0),
            vec![quote!(_0)],
            quote!(#ty),
        )
    } else {
        let mut args = vec![];
        let mut pats = vec![];
        let mut tys = vec![];

        for (i, input) in inputs.iter().enumerate() {
            let i = Ident::new(&format!("_{}", i), Span::call_site());
            let ty = &input.ty;

            args.push(quote!(#i: #ty));

            pats.push(quote!(#i));

            tys.push(quote!(#ty));
        }

        let tupled = {
            let pats = pats.clone();
            quote!((#(#pats,)*))
        };
        let ty = quote!((#(#tys,)*));
        (args, tupled, pats, ty)
    }
}

/// Get the ident for the name of the task
pub fn get_task_name(ctxt: Context, app: &App) -> Ident {
    let s = match ctxt {
        Context::Init => app.init.name.to_string(),
        Context::Idle => app.idle.as_ref().unwrap().name.to_string(),
        Context::HardwareTask(ident) | Context::SoftwareTask(ident) => ident.to_string(),
    };

    Ident::new(&s, Span::call_site())
}

pub fn static_shared_resource_ident(name: &Ident) -> Ident {
    mark_internal_name(&format!("shared_resource_{}", name.to_string()))
}

pub fn static_local_resource_ident(name: &Ident) -> Ident {
    mark_internal_name(&format!("local_resource_{}", name.to_string()))
}

pub fn declared_static_local_resource_ident(name: &Ident, task_name: &Ident) -> Ident {
    mark_internal_name(&format!(
        "local_{}_{}",
        task_name.to_string(),
        name.to_string()
    ))
}
