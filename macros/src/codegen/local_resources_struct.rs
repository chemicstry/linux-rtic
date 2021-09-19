use proc_macro2::TokenStream;
use quote::quote;
use rtic_syntax::{
    ast::{App, TaskLocal},
    Context,
};

use crate::codegen::util;

/// Generates local resources structs
pub fn codegen(ctxt: Context, needs_lt: &mut bool, app: &App) -> TokenStream {
    let mut lt = None;

    let resources = match ctxt {
        Context::Init => &app.init.args.local_resources,
        Context::Idle => &app.idle.as_ref().unwrap().args.local_resources,
        Context::HardwareTask(name) => &app.hardware_tasks[name].args.local_resources,
        Context::SoftwareTask(name) => &app.software_tasks[name].args.local_resources,
    };

    let task_name = util::get_task_name(ctxt, app);

    let mut fields = vec![];
    let mut values = vec![];
    let mut has_cfgs = false;

    for (name, task_local) in resources {
        let (cfgs, ty, is_declared) = match task_local {
            TaskLocal::External => {
                let r = app.local_resources.get(name).expect("UNREACHABLE");
                (&r.cfgs, &r.ty, false)
            }
            TaskLocal::Declared(r) => (&r.cfgs, &r.ty, true),
            _ => unreachable!(),
        };

        has_cfgs |= !cfgs.is_empty();

        let lt = if ctxt.runs_once() {
            quote!('static)
        } else {
            lt = Some(quote!('a));
            quote!('a)
        };

        let mangled_name = if matches!(task_local, TaskLocal::External) {
            util::static_local_resource_ident(name)
        } else {
            util::declared_static_local_resource_ident(name, &task_name)
        };

        fields.push(quote!(
            #(#cfgs)*
            pub #name: &#lt mut #ty
        ));

        let expr = if is_declared {
            // If the local resources is already initialized, we only need to access its value and
            // not go through an `MaybeUninit`
            quote!(#mangled_name.get_mut_unchecked())
        } else {
            quote!(&mut *#mangled_name.get_mut_unchecked().as_mut_ptr())
        };

        values.push(quote!(
            #(#cfgs)*
            #name: #expr
        ));
    }

    if lt.is_some() {
        *needs_lt = true;

        // The struct could end up empty due to `cfg`s leading to an error due to `'a` being unused
        if has_cfgs {
            fields.push(quote!(
                #[doc(hidden)]
                pub __marker__: core::marker::PhantomData<&'a ()>
            ));

            values.push(quote!(__marker__: core::marker::PhantomData))
        }
    }

    let doc = format!("Local resources `{}` has access to", ctxt.ident(app));
    let ident = util::local_resources_ident(ctxt, app);

    quote!(
        #[allow(non_snake_case)]
        #[allow(non_camel_case_types)]
        #[doc = #doc]
        pub struct #ident<#lt> {
            #(#fields,)*
        }

        impl<#lt> #ident<#lt> {
            #[inline(always)]
            pub unsafe fn new() -> Self {
                #ident {
                    #(#values,)*
                }
            }
        }
    )
}
