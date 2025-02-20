use std::collections::HashSet;

use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::Ident;

use crate::utils::generate_doc_comment;
use crate::utils::{format_method_name, generate_doc_comments, Method, Service};

pub struct GenerateTraitService<'a, S> {
    pub service: &'a S,
    pub emit_package: bool,
    pub proto_path: &'a str,
    pub compile_well_known_types: bool,
    pub service_trait: Ident,
    pub disable_comments: &'a HashSet<String>,
    pub use_arc_self: bool,
    pub generate_default_stubs: bool,
}

impl<S: Service> GenerateTraitService<'_, S> {
    pub fn generate_methods(&self) -> TokenStream {
        let mut stream = TokenStream::new();

        for method in self.service.methods() {
            let name = quote::format_ident!("{}", method.name());

            let (req_message, res_message) =
                method.request_response_name(self.proto_path, self.compile_well_known_types);
            let method_doc = if self.disable_comments.contains(&format_method_name(
                self.service,
                method,
                self.emit_package,
            )) {
                TokenStream::new()
            } else {
                generate_doc_comments(method.comment())
            };
            let self_param = if self.use_arc_self {
                quote!(self: std::sync::Arc<Self>)
            } else {
                quote!(&self)
            };
            let not_implemented = quote! {
                Err(kanamaru::Status::unimplemented("Not implemented"))
            };
            let method_tokens: TokenStream = match (
                method.client_streaming(),
                method.server_streaming(),
                self.generate_default_stubs,
            ) {
                (true, true, true) => {
                    let stream =
                        quote::format_ident!("{}Stream", method.identifier().to_upper_camel_case());
                    let stream_doc = generate_doc_comment(format!(
                        " Server streaming response type for the {} method.",
                        method.identifier()
                    ));
                    quote! {
                        #stream_doc
                        type #stream: kanamaru::codegen::tokio_stream::Stream<Item = std::result::Result<IpcMessage<#res_message>, kanamaru::Status>> + std::marker::Send + 'static;

                        #method_doc
                        async fn #name<R: Runtime>(#self_param, request: kanamaru::StreamingRequest<R, #req_message>)
                            -> std::result::Result<kanamaru::StreamingResponse<#res_message, Self::#stream>, kanamaru::Status> {
                            #not_implemented
                        }
                    }
                }
                (true, true, false) => {
                    let stream =
                        quote::format_ident!("{}Stream", method.identifier().to_upper_camel_case());
                    let stream_doc = generate_doc_comment(format!(
                        " Server streaming response type for the {} method.",
                        method.identifier()
                    ));
                    quote! {
                        #stream_doc
                        type #stream: kanamaru::codegen::tokio_stream::Stream<Item = std::result::Result<IpcMessage<#res_message>, kanamaru::Status>> + std::marker::Send + 'static;

                        #method_doc
                        async fn #name<R: Runtime>(#self_param, request: kanamaru::StreamingRequest<R, #req_message>)
                            -> std::result::Result<kanamaru::StreamingResponse<#res_message, Self::#stream>, kanamaru::Status>;
                    }
                }
                (true, false, true) => {
                    quote! {
                        #method_doc
                        async fn #name<R: Runtime>(#self_param, request: kanamaru::StreamingRequest<R, #req_message>)
                            -> std::result::Result<kanamaru::UnaryResponse<#res_message>, kanamaru::Status> {
                            #not_implemented
                        }
                    }
                }
                (true, false, false) => {
                    quote! {
                        #method_doc
                        async fn #name<R: Runtime>(#self_param, request: kanamaru::StreamingRequest<R, #req_message>)
                            -> std::result::Result<kanamaru::UnaryResponse<#res_message>, kanamaru::Status>;
                    }
                }
                (false, true, true) => {
                    let stream =
                        quote::format_ident!("{}Stream", method.identifier().to_upper_camel_case());
                    let stream_doc = generate_doc_comment(format!(
                        " Server streaming response type for the {} method.",
                        method.identifier()
                    ));
                    quote! {
                        #stream_doc
                        type #stream: kanamaru::codegen::tokio_stream::Stream<Item = std::result::Result<IpcMessage<#res_message>, kanamaru::Status>> + std::marker::Send + 'static;

                        #method_doc
                        async fn #name<R: Runtime>(#self_param, request: kanamaru::UnaryRequest<R, #req_message>)
                            -> std::result::Result<kanamaru::StreamingResponse<#res_message, Self::#stream>, kanamaru::Status> {
                            #not_implemented
                        }
                    }
                }
                (false, true, false) => {
                    let stream =
                        quote::format_ident!("{}Stream", method.identifier().to_upper_camel_case());
                    let stream_doc = generate_doc_comment(format!(
                        " Server streaming response type for the {} method.",
                        method.identifier()
                    ));
                    quote! {
                        #stream_doc
                        type #stream: kanamaru::codegen::tokio_stream::Stream<Item = std::result::Result<IpcMessage<#res_message>, kanamaru::Status>> + std::marker::Send + 'static;

                        #method_doc
                        async fn #name<R: Runtime>(#self_param, request: kanamaru::UnaryRequest<R, #req_message>)
                            -> std::result::Result<kanamaru::StreamingResponse<#res_message, Self::#stream>, kanamaru::Status>;
                    }
                }
                (false, false, true) => {
                    quote! {
                        #method_doc
                        async fn #name<R: Runtime>(#self_param, request: kanamaru::UnaryRequest<R, #req_message>)
                            -> std::result::Result<kanamaru::UnaryResponse<#res_message>, kanamaru::Status>{
                            #not_implemented
                        }
                    }
                }
                (false, false, false) => {
                    quote! {
                        #method_doc
                        async fn #name<R: Runtime>(#self_param, request: kanamaru::UnaryRequest<R, #req_message>)
                            -> std::result::Result<kanamaru::UnaryResponse<#res_message>, kanamaru::Status>;
                    }
                }
            };
            stream.extend(method_tokens);
        }
        stream
    }
}

impl<S: Service> ToTokens for GenerateTraitService<'_, S> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let methods = self.generate_methods();
        let trait_doc = generate_doc_comment(format!(
            " Generated trait containing gRPC methods that should be implemented for use with {}Responder.",
            self.service.name()
        ));
        let server_trait = &self.service_trait;
        let _trait = quote! {
            #trait_doc
            #[async_trait]
            pub trait #server_trait : std::marker::Send + std::marker::Sync + 'static {
                #methods
            }
        };
        tokens.extend(_trait);
    }
}
