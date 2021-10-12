use crate::codegen::util;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use rtic_syntax::{analyze::Analysis, ast::App, Context};

pub fn codegen(
    ctxt: Context,
    shared_resources_tick: bool,
    local_resources_tick: bool,
    app: &App,
    _analysis: &Analysis,
) -> TokenStream2 {
    let mut items = vec![];
    let mut module_items = vec![];
    let mut fields = vec![];
    let mut values = vec![];
    // Used to copy task cfgs to the whole module
    let mut task_cfgs = vec![];

    let name = ctxt.ident(app);

    let mut lt = None;
    match ctxt {
        Context::Init => {}

        Context::Idle => {}

        Context::HardwareTask(_) => {}

        Context::SoftwareTask(_) => {}
    }

    // if ctxt.has_locals(app) {
    //     let ident = util::locals_ident(ctxt, app);
    //     module_items.push(quote!(
    //         #[doc(inline)]
    //         pub use super::#ident as Locals;
    //     ));
    // }

    if ctxt.has_local_resources(app) {
        let ident = util::local_resources_ident(ctxt, app);
        let lt = if local_resources_tick {
            lt = Some(quote!('a));
            Some(quote!('a))
        } else {
            None
        };

        module_items.push(quote!(
            #[doc(inline)]
            pub use super::#ident as LocalResources;
        ));

        fields.push(quote!(
            /// Local Resources this task has access to
            pub local: #name::LocalResources<#lt>
        ));

        values.push(quote!(local: #name::LocalResources::new()));
    }

    if ctxt.has_shared_resources(app) {
        let ident = util::shared_resources_ident(ctxt, app);
        let lt = if shared_resources_tick {
            lt = Some(quote!('a));
            Some(quote!('a))
        } else {
            None
        };

        module_items.push(quote!(
            #[doc(inline)]
            pub use super::#ident as SharedResources;
        ));

        fields.push(quote!(
            /// Shared Resources this task has access to
            pub shared: #name::SharedResources<#lt>
        ));

        let marker = if ctxt.is_init() {
            None
        } else {
            Some(quote!(marker))
        };
        values.push(quote!(shared: #name::SharedResources::new(#marker)));
    }

    if let Context::Init = ctxt {
        let monotonic_types: Vec<_> = app
            .monotonics
            .iter()
            .map(|(_, monotonic)| {
                let mono = &monotonic.ty;
                quote! {#mono}
            })
            .collect();

        let internal_monotonics_ident = util::mark_internal_name("Monotonics");

        items.push(quote!(
            /// Monotonics used by the system
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            pub struct #internal_monotonics_ident(
                #(pub #monotonic_types),*
            );
        ));

        module_items.push(quote!(
            pub use super::#internal_monotonics_ident as Monotonics;
        ));
    }

    let doc = match ctxt {
        Context::Idle => "Idle loop",
        Context::Init => "Initialization function",
        Context::HardwareTask(_) => "Hardware task",
        Context::SoftwareTask(_) => "Software task",
    };

    let v = Vec::new();
    let cfgs = match ctxt {
        Context::HardwareTask(t) => {
            &app.hardware_tasks[t].cfgs
            // ...
        }
        Context::SoftwareTask(t) => {
            &app.software_tasks[t].cfgs
            // ...
        }
        _ => &v,
    };

    let marker = if ctxt.is_init() {
        None
    } else {
        Some(quote!(marker: &#lt core::marker::PhantomData<()>))
    };

    let internal_context_name = util::internal_task_context_ident(name);

    items.push(quote!(
        #(#cfgs)*
        /// Execution context
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        pub struct #internal_context_name<#lt> {
            #(#fields,)*
        }

        #(#cfgs)*
        impl<#lt> #internal_context_name<#lt> {
            #[inline(always)]
            pub unsafe fn new(#marker) -> Self {
                #internal_context_name {
                    #(#values,)*
                }
            }
        }
    ));

    module_items.push(quote!(
        #(#cfgs)*
        pub use super::#internal_context_name as Context;
    ));

    if let Context::SoftwareTask(..) = ctxt {
        let spawnee = &app.software_tasks[name];
        let priority = spawnee.args.priority;
        let spawn_enum = util::spawn_enum_ident(priority);
        let cfgs = &spawnee.cfgs;
        // Store a copy of the task cfgs
        task_cfgs = cfgs.clone();
        let (inputs_args, inputs_tupled, inputs_untupled, inputs_ty) =
            util::regroup_inputs(&spawnee.inputs);
        let run_queue = util::run_queue_ident(priority);
        let input_queue = util::task_input_queue_ident(name);

        let internal_spawn_ident = util::internal_task_spawn_ident(name);

        // Spawn caller
        items.push(quote!(
            #(#cfgs)*
            /// Spawns the task directly
            pub fn #internal_spawn_ident(#(#inputs_args,)*) -> Result<(), #inputs_ty> {
                let input = #inputs_tupled;

                match #input_queue.insert(input) {
                    Ok(handle) => {
                        #[cfg(feature = "profiling")]
                        rtic::tracing::trace!("spawn {}", stringify!(#name));

                        // Should never fail if capacity calculations are correct
                        if #run_queue.0.send((#spawn_enum::#name, handle)).is_err() {
                            panic!("Run queue full!");
                        }

                        Ok(())
                    },
                    Err(input) => Err(input)
                }
            }
        ));

        let internal_spawn_at_ident = util::internal_task_spawn_at_ident(name);

        // Spawn at caller
        items.push(quote!(
            #(#cfgs)*
            /// Spawns the task directly
            pub fn #internal_spawn_at_ident(instant: std::time::Instant, #(#inputs_args,)*) -> Result<(), #inputs_ty> {
                let input = #inputs_tupled;

                match #input_queue.insert(input) {
                    Ok(handle) => {
                        #[cfg(feature = "profiling")]
                        rtic::tracing::trace!("schedule {} at {:?}", stringify!(#name), instant);

                        // Should never fail if capacity calculations are correct
                        if #run_queue.0.send_scheduled((#spawn_enum::#name, handle), instant).is_err() {
                            panic!("Schedule queue full!");
                        }

                        Ok(())
                    },
                    Err(input) => Err(input)
                }
            }
        ));

        let internal_spawn_after_ident = util::internal_task_spawn_after_ident(name);

        // Spawn after caller
        items.push(quote!(
            #(#cfgs)*
            /// Spawns the task directly
            pub fn #internal_spawn_after_ident(dur: std::time::Duration, #(#inputs_args,)*) -> Result<(), #inputs_ty> {
                let instant = std::time::Instant::now() + dur;

                #[cfg(feature = "profiling")]
                rtic::tracing::trace!("schedule {} after {:?}", stringify!(#name), dur);

                #internal_spawn_at_ident(instant #(,#inputs_untupled)*)
            }
        ));

        module_items.push(quote!(
            #(#cfgs)*
            pub use super::#internal_spawn_ident as spawn;
            pub use super::#internal_spawn_at_ident as spawn_at;
            pub use super::#internal_spawn_after_ident as spawn_after;
        ));
    }

    if !items.is_empty() {
        quote!(
            #(#items)*

            #[allow(non_snake_case)]
            #(#task_cfgs)*
            #[doc = #doc]
            pub mod #name {
                #(#module_items)*
            }
        )
    } else {
        quote!()
    }
}
