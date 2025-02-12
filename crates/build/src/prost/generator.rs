use proc_macro2::TokenStream;
use prost_build::ServiceGenerator as SVCGen;
use quote::ToTokens;

use super::{codegen::CodegenResponderBuilder, ProstBuilder};

pub struct ServiceGenerator {
    builder: ProstBuilder,
    responder: TokenStream,
}

impl ServiceGenerator {
    pub fn new(builder: ProstBuilder) -> Self {
        Self {
            builder,
            responder: TokenStream::new(),
        }
    }
}

impl SVCGen for ServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, _buf: &mut String) {
        let responder = CodegenResponderBuilder {
            service: &service,
            emit_package: self.builder.emit_package,
            proto_path: &self.builder.proto_path,
            compile_well_known_types: self.builder.compile_well_known_types,
            use_arc_self: self.builder.use_arc_self,
            generate_default_stubs: self.builder.generate_default_stubs,
            attributes: &self.builder.responder_attributes,
            disabled_comments: &self.builder.disable_comments,
        };

        self.responder.extend(responder.into_token_stream());
    }
    fn finalize(&mut self, buf: &mut String) {
        if !self.responder.is_empty() {
            let responder = &self.responder;

            let responder_service = quote::quote! {
                #responder
            };

            let ast: syn::File = syn::parse2(responder_service).expect("not a valid tokenstream");
            let code = prettyplease::unparse(&ast);
            buf.push_str(&code);

            self.responder = TokenStream::default();
        }
    }
}
