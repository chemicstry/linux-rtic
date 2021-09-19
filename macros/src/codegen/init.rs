use proc_macro2::TokenStream;
use quote::quote;
use rtic_syntax::{Context, analyze::Analysis, ast::App};

use crate::{codegen::{local_resources_struct, module}};

type CodegenResult = (
    // all generated init definitions
    Vec<TokenStream>,
    // call_init -- the call to the user `#[init]`
    TokenStream,
);

/// Generates support code for `#[init]` functions
pub fn codegen(app: &App, analysis: &Analysis) -> CodegenResult {
    let init = &app.init;
    let mut local_needs_lt = false;
    let name = &init.name;

    // init function type definitions
    let mut defs = vec![];

    let context = &init.context;
    let attrs = &init.attrs;
    let stmts = &init.stmts;
    let shared = &init.user_shared_struct;
    let local = &init.user_local_struct;

    let shared_resources: Vec<_> = app
        .shared_resources
        .iter()
        .map(|(k, v)| {
            let ty = &v.ty;
            let cfgs = &v.cfgs;
            quote!(
                #(#cfgs)*
                #k: #ty,
            )
        })
        .collect();
    let local_resources: Vec<_> = app
        .local_resources
        .iter()
        .map(|(k, v)| {
            let ty = &v.ty;
            let cfgs = &v.cfgs;
            quote!(
                #(#cfgs)*
                #k: #ty,
            )
        })
        .collect();
    defs.push(quote! {
        struct #shared {
            #(#shared_resources)*
        }

        struct #local {
            #(#local_resources)*
        }
    });

    // let locals_pat = locals_pat.iter();

    let user_init_return = quote! {#shared, #local, #name::Monotonics};

    defs.push(quote!(
        #(#attrs)*
        #[allow(non_snake_case)]
        fn #name(#context: #name::Context) -> (#user_init_return) {
            #(#stmts)*
        }
    ));

    // `${task}Locals`
    if !init.args.local_resources.is_empty() {
        let item = local_resources_struct::codegen(Context::Init, &mut local_needs_lt, app);

        defs.push(item);
    }

    defs.push(module::codegen(
        Context::Init,
        false,
        local_needs_lt,
        app,
        analysis,
    ));

    // let locals_new = locals_new.iter();
    let call_init = quote! {
        let (shared_resources, local_resources, mut monotonics) = #name(#name::Context::new());
    };

    (defs, call_init)
}
