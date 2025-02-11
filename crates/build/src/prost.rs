pub mod codegen;

use proc_macro2::TokenStream;
use prost_build::{Method as PMethod, Service as PService};
use quote::ToTokens;

use crate::utils::{Method, Service};

impl Service for PService {
    type Comment = String;
    type Method = PMethod;

    fn name(&self) -> &str {
        &self.name
    }

    fn package(&self) -> &str {
        &self.package
    }

    fn identifier(&self) -> &str {
        &self.proto_name
    }

    fn methods(&self) -> &[Self::Method] {
        &self.methods
    }

    fn comment(&self) -> &[Self::Comment] {
        &self.comments.leading[..]
    }
}

const NON_PATH_TYPE_ALLOWLIST: &[&str] = &["()"];

impl Method for PMethod {
    type Comment = String;

    fn name(&self) -> &str {
        &self.name
    }

    fn identifier(&self) -> &str {
        &self.proto_name
    }

    fn client_streaming(&self) -> bool {
        self.client_streaming
    }

    fn server_streaming(&self) -> bool {
        self.server_streaming
    }

    fn comment(&self) -> &[Self::Comment] {
        &self.comments.leading[..]
    }

    fn request_response_name(
        &self,
        proto_path: &str,
        compile_well_known_types: bool,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        let request = convert_type(
            compile_well_known_types,
            proto_path,
            &self.input_proto_type,
            &self.input_type,
        );
        let response = convert_type(
            compile_well_known_types,
            proto_path,
            &self.output_proto_type,
            &self.output_type,
        );
        (request, response)
    }
}

fn convert_type(
    compile_well_known_types: bool,
    proto_path: &str,
    proto_type: &str,
    rust_type: &str,
) -> TokenStream {
    if (is_google_type(proto_type) && !compile_well_known_types)
        || rust_type.starts_with("::")
        || NON_PATH_TYPE_ALLOWLIST.iter().any(|ty| *ty == rust_type)
    {
        rust_type.parse::<TokenStream>().unwrap()
    } else if rust_type.starts_with("crate::") {
        syn::parse_str::<syn::Path>(rust_type)
            .unwrap()
            .to_token_stream()
    } else {
        syn::parse_str::<syn::Path>(&format!("{}::{}", proto_path, rust_type))
            .unwrap()
            .to_token_stream()
    }
}

fn is_google_type(ty: &str) -> bool {
    ty.starts_with(".google.protobuf")
}
