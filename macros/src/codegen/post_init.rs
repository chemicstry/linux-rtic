use proc_macro2::TokenStream;
use quote::quote;
use rtic_syntax::{
    analyze::{Analysis, Ownership},
    ast::App,
};

use crate::codegen::util;

/// Generates code that runs after `#[init]` returns
pub fn codegen(app: &App, analysis: &Analysis) -> Vec<TokenStream> {
    let mut stmts = vec![];

    // Initialize all lazy_static queues
    for (name, _task) in &app.software_tasks {
        let tiq_ident = util::task_input_queue_ident(name);
        stmts.push(quote!(
            rtic::export::lazy_static::initialize(&#tiq_ident);
        ));
    }

    // Initialize shared resources
    for (name, res) in &app.shared_resources {
        let mangled_name = util::static_shared_resource_ident(name);
        // If it's live
        let cfgs = res.cfgs.clone();
        if analysis.shared_resource_locations.get(name).is_some() {
            let ceiling = match analysis.ownerships.get(name) {
                Some(Ownership::Owned { priority }) => *priority,
                Some(Ownership::CoOwned { priority }) => *priority,
                Some(Ownership::Contended { ceiling }) => *ceiling,
                None => 0,
            };

            stmts.push(quote!(
                // We include the cfgs
                #(#cfgs)*
                // Resource is a RacyCell<MaybeUninit<T>>
                // - `get_mut_unchecked` to obtain `MaybeUninit<T>`
                // - `as_mut_ptr` to obtain a raw pointer to `MaybeUninit<T>`
                // - `write` the defined value for the late resource T
                #mangled_name.get_mut_unchecked().as_mut_ptr().write(rtic::export::PcpMutex::new(shared_resources.#name, #ceiling));
            ));
        }
    }

    // Initialize local resources
    for (name, res) in &app.local_resources {
        let mangled_name = util::static_local_resource_ident(name);
        // If it's live
        let cfgs = res.cfgs.clone();
        if analysis.local_resource_locations.get(name).is_some() {
            stmts.push(quote!(
                // We include the cfgs
                #(#cfgs)*
                // Resource is a RacyCell<MaybeUninit<T>>
                // - `get_mut_unchecked` to obtain `MaybeUninit<T>`
                // - `as_mut_ptr` to obtain a raw pointer to `MaybeUninit<T>`
                // - `write` the defined value for the late resource T
                #mangled_name.get_mut_unchecked().as_mut_ptr().write(local_resources.#name);
            ));
        }
    }

    stmts
}
