use proc_macro2::TokenStream;
use quote::quote;
use rtic_syntax::{analyze::Analysis, ast::App};

use crate::codegen::util;

/// Generates `static` variables and shared resource proxies
pub fn codegen(
    app: &App,
    _analysis: &Analysis,
) -> (
    // mod_app -- the `static` variables behind the proxies
    Vec<TokenStream>,
    // mod_resources -- the `resources` module
    TokenStream,
) {
    let mut mod_app = vec![];
    let mut mod_resources = vec![];

    for (name, res) in &app.shared_resources {
        let cfgs = &res.cfgs;
        let ty = &res.ty;
        let mangled_name = &util::static_shared_resource_ident(&name);
        let attrs = &res.attrs;

        // For future use
        // let doc = format!(" RTIC internal: {}:{}", file!(), line!());
        mod_app.push(quote!(
            #[allow(non_camel_case_types)]
            #[allow(non_upper_case_globals)]
            // #[doc = #doc]
            #[doc(hidden)]
            #(#attrs)*
            #(#cfgs)*
            static #mangled_name: rtic::RacyCell<core::mem::MaybeUninit<rtic::export::PcpMutex<#ty>>>
             = rtic::RacyCell::new(core::mem::MaybeUninit::uninit());
        ));

        // For future use
        // let doc = format!(" RTIC internal: {}:{}", file!(), line!());

        if !res.properties.lock_free {
            mod_resources.push(quote!(
                // #[doc = #doc]
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                #(#cfgs)*
                pub struct #name<'a> {
                    thread_state: &'a ThreadState,
                }

                #(#cfgs)*
                impl<'a> #name<'a> {
                    #[inline(always)]
                    pub unsafe fn new(thread_state: &'a ThreadState) -> Self {
                        #name { thread_state }
                    }

                    #[inline(always)]
                    pub fn thread_state(&self) -> &ThreadState {
                        self.thread_state
                    }
                }
            ));

            let ptr = quote!(
                #(#cfgs)*
                #mangled_name.get_mut_unchecked().as_mut_ptr()
            );

            let tracing_name = format!("shared_{}", name);
            let tracing_name_locked = format!("shared_{}_locked", name);

            mod_app.push(quote!(
                #(#cfgs)*
                impl<'a> rtic::Mutex for shared_resources::#name<'a> {
                    type T = #ty;

                    #[inline(always)]
                    fn lock<RTIC_INTERNAL_R>(&mut self, f: impl FnOnce(&mut #ty) -> RTIC_INTERNAL_R) -> RTIC_INTERNAL_R {
                        let mutex = unsafe { & *#ptr };

                        #[cfg(feature = "profiling")]
                        let _span = rtic::tracing::span!(rtic::tracing::Level::TRACE, #tracing_name).entered();

                        #[cfg(feature = "profiling")]
                        rtic::tracing::trace!("locking");

                        let r = mutex.lock(self.thread_state(), |res| {
                            #[cfg(feature = "profiling")]
                            let _span = rtic::tracing::span!(rtic::tracing::Level::TRACE, #tracing_name_locked).entered();

                            #[cfg(feature = "profiling")]
                            rtic::tracing::trace!("locked");

                            // Execute user closure with the resource reference
                            let r = f(res);

                            #[cfg(feature = "profiling")]
                            rtic::tracing::trace!("unlocking");

                            r
                        });

                        #[cfg(feature = "profiling")]
                        rtic::tracing::trace!("unlocked");

                        r
                    }
                }
            ));
        }
    }

    let mod_resources = if mod_resources.is_empty() {
        quote!()
    } else {
        quote!(mod shared_resources {
            use rtic::export::ThreadState;

            #(#mod_resources)*
        })
    };

    (mod_app, mod_resources)
}
