use std::collections::HashSet;

use quote::{quote, ToTokens};
use responder::GenerateResponderService;
use traits::GenerateTraitService;

use crate::utils::{format_service_name, naive_snake_case, Attributes, Service};

pub mod responder;
pub mod traits;

pub struct CodegenResponderBuilder<'a, S> {
    pub service: &'a S,
    pub emit_package: bool,
    pub proto_path: &'a str,
    pub compile_well_known_types: bool,
    pub use_arc_self: bool,
    pub generate_default_stubs: bool,
    pub attributes: &'a Attributes,
    pub disabled_comments: &'a HashSet<String>,
}

impl<S: Service> ToTokens for CodegenResponderBuilder<'_, S> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let service_responder = quote::format_ident!("{}Responder", self.service.name());
        let service_trait = quote::format_ident!("{}", self.service.name());
        let service_mod = quote::format_ident!("{}_server", naive_snake_case(self.service.name()));
        let package = if self.emit_package {
            self.service.package()
        } else {
            ""
        };
        let service_name = format_service_name(self.service, self.emit_package);
        let mod_attributes = self.attributes.for_mod(package);

        let _trait = GenerateTraitService {
            service: self.service,
            emit_package: self.emit_package,
            proto_path: self.proto_path,
            compile_well_known_types: self.compile_well_known_types,
            service_trait: service_trait.clone(),
            disable_comments: self.disabled_comments,
            use_arc_self: self.use_arc_self,
            generate_default_stubs: self.generate_default_stubs,
        };
        let responder = GenerateResponderService {
            service: self.service,
            emit_package: self.emit_package,
            proto_path: self.proto_path,
            compile_well_known_types: self.compile_well_known_types,
            use_arc_self: self.use_arc_self,
            generate_default_stubs: self.generate_default_stubs,
            responder_service: service_responder,
            service_trait,
            attributes: self.attributes.for_struct(&service_name),
            disabled_comments: self.disabled_comments,
        };
        let res = quote! {
            /// Generated responder implementations.
            #(#mod_attributes)*
            pub mod #service_mod {
                #![allow(
                    unused_variables,
                    dead_code,
                    missing_docs,
                    clippy::wildcard_imports
                )]
                use tonic::codegen::*;

                #_trait

                #responder
            }
        };
        tokens.extend(res);
    }
}
