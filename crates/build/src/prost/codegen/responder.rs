use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Ident;

use crate::utils::{
    format_method_name, format_method_path, format_service_name, generate_doc_comments, Method,
    Service,
};

pub struct GenerateResponderService<'a, S> {
    pub service: &'a S,
    pub emit_package: bool,
    pub proto_path: &'a str,
    pub compile_well_known_types: bool,
    pub use_arc_self: bool,
    pub generate_default_stubs: bool,
    pub responder_service: Ident,
    pub service_trait: Ident,
    pub attributes: Vec<syn::Attribute>,
    pub disabled_comments: &'a HashSet<String>,
}

impl<S: Service> GenerateResponderService<'_, S> {
    fn impl_service_name(&self) -> TokenStream {
        let name = self.service.name().to_string();
        quote! {
            fn service_name(&self) -> String {
                #name
            }
        }
    }
    fn impl_get_rpc_type(&self) -> TokenStream {
        let methods = self
            .service
            .methods()
            .iter()
            .map(|method| {
                let name_method = format_method_name(self.service, method, self.emit_package);
                let path_method = format_method_path(self.service, method, self.emit_package);
                let _type = match (method.client_streaming(), method.server_streaming()) {
                    (true, true) => quote! {
                        RPCType::Duplex
                    },
                    (true, false) => quote! {
                        RPCType::ClientStreaming
                    },
                    (false, true) => quote! {
                        RPCType::ServerStreaming
                    },
                    (false, false) => quote! {
                        RPCType::Unary
                    },
                };
                quote! {
                    #name_method => Some(#_type),
                    #path_method => Some(#_type),
                }
            })
            .reduce(|mut acc, t| {
                acc.extend(t);
                acc
            });
        quote! {
            fn get_rpc_type(&self, route: &str) -> Option<RPCType> {
                match route {
                    #methods
                    _ => None
                }
            }
        }
    }
    fn impl_respond(&self) -> TokenStream {
        let inner_arg = if self.use_arc_self {
            quote!(inner)
        } else {
            quote!(&inner)
        };
        let service_trait = &self.service_trait;
        let methods = self
            .service
            .methods()
            .iter()
            .map(|method| {
                let method_name = quote::format_ident!("{}", method.name());
                let name_method = format_method_name(self.service, method, self.emit_package);
                let path_method = format_method_path(self.service, method, self.emit_package);
                let (request, response) = method.request_response_name(self.proto_path, self.compile_well_known_types);
                let resp = match (method.client_streaming(), method.server_streaming()) {
                    (true, true) => {
                        quote! {
                            let request: StreamingRequest<R, _> = StreamingRequest<R, #request>::new(wv_cancel_token.clone(), message.clone());
                            let mut handle = spawn(async move {
                                let resp = <T as #service_trait>::#method_name(#inner_arg, request).await?;
                                res.send_responses().await.map_err(|err| Status::internal(err))?;
                                Ok::<(), Status>()
                            });
                            if cancel {
                                let cancel_token = wv_cancel_token.token();
                                let _abort = handle.abort_handle();
                                select! {
                                    _ = cancel_token.cancelled() => {
                                        Err(Status::aborted("Aborted task"))
                                    },
                                    res = &mut task {
                                        if let Err(err) = res.map_err(|err| Status::internal(res)).and_then(|maybe_res| maybe_res) {
                                            Err(err.into())
                                        }else{
                                            Ok(None::<IpcMessageBase>)
                                        }
                                    }
                                }
                            }else {
                                if let Err(err) = task.await.map_err(|err| Status::internal(res)).and_then(|maybe_res| maybe_res) {
                                    Err(err.into())
                                }else{
                                    Ok(None::<IpcMessageBase>)
                                }
                            }
                        }
                    },
                    (true, false) => {
                        quote! {
                            let request: StreamingRequest<R, _> = StreamingRequest<R, #request>::new(wv_cancel_token.clone(), message.clone());
                            let mut handle = spawn(async move {
                                let resp = <T as #service_trait>::#method_name(#inner_arg, request).await?;
                                Ok::<#response, Status>(resp)
                            });
                            if cancel {
                                let cancel_token = wv_cancel_token.token();
                                let _abort = handle.abort_handle();
                                select! {
                                    _ = cancel_token.cancelled() => {
                                        Err(Status::aborted("Aborted task"))
                                    },
                                    res = &mut task {
                                        Ok(res.map_err(|err| Status::internal(res)).and_then(|maybe_res| maybe_res)?)
                                    }
                                }
                            }else {
                                Ok(res.map_err(|err| Status::internal(res)).and_then(|maybe_res| maybe_res)?)
                            }
                        }
                    },
                    (false, true) => quote! {
                        quote! {
                            let request: UnaryRequest<R, _> = UnaryRequest<R, #request>::new(wv_cancel_token.clone(), message.clone());
                            let mut handle = spawn(async move {
                                let resp = <T as #service_trait>::#method_name(#inner_arg, request).await?;
                                res.send_responses().await.map_err(|err| Status::internal(err))?;
                                Ok::<(), Status>()
                            });
                            if cancel {
                                let cancel_token = wv_cancel_token.token();
                                let _abort = handle.abort_handle();
                                select! {
                                    _ = cancel_token.cancelled() => {
                                        Err(Status::aborted("Aborted task"))
                                    },
                                    res = &mut task {
                                        if let Err(err) = res.map_err(|err| Status::internal(res)).and_then(|maybe_res| maybe_res) {
                                            Err(err.into())
                                        }else{
                                            Ok(None::<IpcMessageBase>)
                                        }
                                    }
                                }
                            }else {
                                if let Err(err) = task.await.map_err(|err| Status::internal(res)).and_then(|maybe_res| maybe_res) {
                                    Err(err.into())
                                }else{
                                    Ok(None::<IpcMessageBase>)
                                }
                            }
                        }
                    },
                    (false, false) => quote! {
                        quote! {
                            let request: UnaryRequest<R, _> = UnaryRequest<R, #request>::new(wv_cancel_token.clone(), message.clone());
                            let mut handle = spawn(async move {
                                let resp = <T as #service_trait>::#method_name(#inner_arg, request).await?;
                                Ok::<#response, Status>(resp)
                            });
                            if cancel {
                                let cancel_token = wv_cancel_token.token();
                                let _abort = handle.abort_handle();
                                select! {
                                    _ = cancel_token.cancelled() => {
                                        Err(Status::aborted("Aborted task"))
                                    },
                                    res = &mut task {
                                        Ok(res.map_err(|err| Status::internal(res)).and_then(|maybe_res| maybe_res)?)
                                    }
                                }
                            }else {
                                Ok(res.map_err(|err| Status::internal(res)).and_then(|maybe_res| maybe_res)?)
                            }
                        }
                    },
                };
                quote! {
                    #name_method => {
                        #resp
                    },
                    #path_method => {
                        #resp
                    },
                }
            })
            .reduce(|mut acc, t| {
                acc.extend(t);
                acc
            });
        quote! {
            fn respond(&self, message: RawRequest, webview: Webview<R>, resovler: InvokeResolver<R>) {
                let inner = self.inner.clone();
                let cancel = self.cancel;
                let wv_cancel_token = Arc::new(message.cancel_token(webview.clone()));
                resovler.respond_async::<Option<IpcMessageBase>, _>(async move {
                    match &message.route {
                        #methods
                        _ => Err(Status::not_found(format!("this route {} is not found", message.route)).into())
                    }
                })
            }
        }
    }
    fn impl_responder(&self) -> TokenStream {
        let responder_service = &self.responder_service;
        let service_trait = &self.service_trait;
        let rpc_type = self.impl_get_rpc_type();
        let service_name = self.impl_service_name();
        let respond = self.impl_respond();
        quote! {
            impl<T, R> Responder<R> for #responder_service<T>
                where
                    T: #service_trait,
                    R: Runtime
            {
                #rpc_type
                #service_name
                #respond
            }
        }
    }
}

impl<S: Service> ToTokens for GenerateResponderService<'_, S> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let responder_service = &self.responder_service;
        let impl_responder = self.impl_responder();

        let service_name = format_service_name(self.service, self.emit_package);

        let service_doc = if self.disabled_comments.contains(&service_name) {
            TokenStream::new()
        } else {
            generate_doc_comments(self.service.comment())
        };
        let struct_attributes = &self.attributes;

        let token = quote! {
            #service_doc
            #(#struct_attributes)*
            #[derive(Debug)]
            pub struct #responder_service<T, R> {
                inner: Arc<T>,
                cancel: bool,
                runtime: PhantomData<R>
            }
            impl<T, R> #responder_service<T, R> {
                pub fn from_arc(inner: Arc<T>) -> Self {
                    Self {
                        inner,
                        cancel: true
                    }
                }
                pub fn new(inner: T) -> Self {
                    Self::from_arc(Arc::new(T))
                }
                pub fn cancel(mut self, cancel: bool) -> Self {
                    self.cancel = cancel;
                    self
                }
            }
            #impl_responder
        };
        tokens.extend(token);
    }
}
