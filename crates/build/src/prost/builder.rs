use std::{
    collections::HashSet,
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::utils::Attributes;

pub struct ProstBuilder {
    pub(crate) file_descriptor_set_path: Option<PathBuf>,
    pub(crate) skip_protoc_run: bool,
    pub(crate) extern_path: Vec<(String, String)>,
    pub(crate) field_attributes: Vec<(String, String)>,
    pub(crate) type_attributes: Vec<(String, String)>,
    pub(crate) message_attributes: Vec<(String, String)>,
    pub(crate) enum_attributes: Vec<(String, String)>,
    pub(crate) boxed: Vec<String>,
    pub(crate) btree_map: Option<Vec<String>>,
    pub(crate) bytes: Option<Vec<String>>,
    pub(crate) responder_attributes: Attributes,
    pub(crate) proto_path: String,
    pub(crate) emit_package: bool,
    pub(crate) compile_well_known_types: bool,
    pub(crate) protoc_args: Vec<OsString>,
    pub(crate) include_file: Option<PathBuf>,
    pub(crate) emit_rerun_if_changed: bool,
    pub(crate) disable_comments: HashSet<String>,
    pub(crate) use_arc_self: bool,
    pub(crate) generate_default_stubs: bool,
    pub(crate) skip_debug: HashSet<String>,

    out_dir: Option<PathBuf>,
}

impl ProstBuilder {
    /// Generate a file containing the encoded `prost_types::FileDescriptorSet` for protocol buffers
    /// modules. This is required for implementing gRPC Server Reflection.
    pub fn file_descriptor_set_path(mut self, path: impl AsRef<Path>) -> Self {
        self.file_descriptor_set_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// In combination with with file_descriptor_set_path, this can be used to provide a file
    /// descriptor set as an input file, rather than having prost-build generate the file by
    /// calling protoc.
    pub fn skip_protoc_run(mut self) -> Self {
        self.skip_protoc_run = true;
        self
    }

    /// Set the output directory to generate code to.
    ///
    /// Defaults to the `OUT_DIR` environment variable.
    pub fn out_dir(mut self, out_dir: impl AsRef<Path>) -> Self {
        self.out_dir = Some(out_dir.as_ref().to_path_buf());
        self
    }

    /// Declare externally provided Protobuf package or type.
    ///
    /// Passed directly to `prost_build::Config.extern_path`.
    /// Note that both the Protobuf path and the rust package paths should both be fully qualified.
    /// i.e. Protobuf paths should start with "." and rust paths should start with "::"
    pub fn extern_path(mut self, proto_path: impl AsRef<str>, rust_path: impl AsRef<str>) -> Self {
        self.extern_path.push((
            proto_path.as_ref().to_string(),
            rust_path.as_ref().to_string(),
        ));
        self
    }

    /// Add additional attribute to matched messages, enums, and one-offs.
    ///
    /// Passed directly to `prost_build::Config.field_attribute`.
    pub fn field_attribute<P: AsRef<str>, A: AsRef<str>>(mut self, path: P, attribute: A) -> Self {
        self.field_attributes
            .push((path.as_ref().to_string(), attribute.as_ref().to_string()));
        self
    }

    /// Add additional attribute to matched messages, enums, and one-offs.
    ///
    /// Passed directly to `prost_build::Config.type_attribute`.
    pub fn type_attribute<P: AsRef<str>, A: AsRef<str>>(mut self, path: P, attribute: A) -> Self {
        self.type_attributes
            .push((path.as_ref().to_string(), attribute.as_ref().to_string()));
        self
    }

    /// Add additional attribute to matched messages.
    ///
    /// Passed directly to `prost_build::Config.message_attribute`.
    pub fn message_attribute<P: AsRef<str>, A: AsRef<str>>(
        mut self,
        path: P,
        attribute: A,
    ) -> Self {
        self.message_attributes
            .push((path.as_ref().to_string(), attribute.as_ref().to_string()));
        self
    }

    /// Add additional attribute to matched enums.
    ///
    /// Passed directly to `prost_build::Config.enum_attribute`.
    pub fn enum_attribute<P: AsRef<str>, A: AsRef<str>>(mut self, path: P, attribute: A) -> Self {
        self.enum_attributes
            .push((path.as_ref().to_string(), attribute.as_ref().to_string()));
        self
    }

    /// Add additional boxed fields.
    ///
    /// Passed directly to `prost_build::Config.boxed`.
    pub fn boxed<P: AsRef<str>>(mut self, path: P) -> Self {
        self.boxed.push(path.as_ref().to_string());
        self
    }

    /// Configure the code generator to generate Rust `BTreeMap` fields for Protobuf `map` type
    /// fields.
    ///
    /// Passed directly to `prost_build::Config.btree_map`.
    ///
    /// Note: previous configured paths for `btree_map` will be cleared.
    pub fn btree_map<I, S>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.btree_map = Some(
            paths
                .into_iter()
                .map(|path| path.as_ref().to_string())
                .collect(),
        );
        self
    }

    /// Configure the code generator to generate Rust `bytes::Bytes` fields for Protobuf `bytes`
    /// type fields.
    ///
    /// Passed directly to `prost_build::Config.bytes`.
    ///
    /// Note: previous configured paths for `bytes` will be cleared.
    pub fn bytes<I, S>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.bytes = Some(
            paths
                .into_iter()
                .map(|path| path.as_ref().to_string())
                .collect(),
        );
        self
    }

    /// Add additional attribute to matched responder `mod`s. Matches on the package name.
    pub fn responder_mod_attribute<P: AsRef<str>, A: AsRef<str>>(
        mut self,
        path: P,
        attribute: A,
    ) -> Self {
        self.responder_attributes
            .push_mod(path.as_ref().to_string(), attribute.as_ref().to_string());
        self
    }

    /// Add additional attribute to matched responder servers. Matches on the service name.
    pub fn responder_attribute<P: AsRef<str>, A: AsRef<str>>(
        mut self,
        path: P,
        attribute: A,
    ) -> Self {
        self.responder_attributes
            .push_struct(path.as_ref().to_string(), attribute.as_ref().to_string());
        self
    }

    /// Set the path to where tonic will search for the Request/Response proto structs
    /// live relative to the module where you call `include_proto!`.
    ///
    /// This defaults to `super` since tonic will generate code in a module.
    pub fn proto_path(mut self, proto_path: impl AsRef<str>) -> Self {
        self.proto_path = proto_path.as_ref().to_string();
        self
    }

    /// Configure Prost `protoc_args` build arguments.
    ///
    /// Note: Enabling `--experimental_allow_proto3_optional` requires protobuf >= 3.12.
    pub fn protoc_arg<A: AsRef<str>>(mut self, arg: A) -> Self {
        self.protoc_args.push(arg.as_ref().into());
        self
    }

    /// Disable service and rpc comments emission.
    pub fn disable_comments(mut self, path: impl AsRef<str>) -> Self {
        self.disable_comments.insert(path.as_ref().to_string());
        self
    }

    /// Emit `Arc<Self>` receiver type in server traits instead of `&self`.
    pub fn use_arc_self(mut self, enable: bool) -> Self {
        self.use_arc_self = enable;
        self
    }

    /// Emits GRPC endpoints with no attached package. Effectively ignores protofile package declaration from grpc context.
    ///
    /// This effectively sets prost's exported package to an empty string.
    pub fn disable_package_emission(mut self) -> Self {
        self.emit_package = false;
        self
    }

    /// Enable or disable directing Prost to compile well-known protobuf types instead
    /// of using the already-compiled versions available in the `prost-types` crate.
    ///
    /// This defaults to `false`.
    pub fn compile_well_known_types(mut self, compile_well_known_types: bool) -> Self {
        self.compile_well_known_types = compile_well_known_types;
        self
    }

    /// Configures the optional module filename for easy inclusion of all generated Rust files
    ///
    /// If set, generates a file (inside the `OUT_DIR` or `out_dir()` as appropriate) which contains
    /// a set of `pub mod XXX` statements combining to load all Rust files generated.  This can allow
    /// for a shortcut where multiple related proto files have been compiled together resulting in
    /// a semi-complex set of includes.
    pub fn include_file(mut self, path: impl AsRef<Path>) -> Self {
        self.include_file = Some(path.as_ref().to_path_buf());
        self
    }

    /// Enable or disable emitting
    /// [`cargo:rerun-if-changed=PATH`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed)
    /// instructions for Cargo.
    ///
    /// If set, writes instructions to `stdout` for Cargo so that it understands
    /// when to rerun the build script. By default, this setting is enabled if
    /// the `CARGO` environment variable is set. The `CARGO` environment
    /// variable is set by Cargo for build scripts. Therefore, this setting
    /// should be enabled automatically when run from a build script. However,
    /// the method of detection is not completely reliable since the `CARGO`
    /// environment variable can have been set by anything else. If writing the
    /// instructions to `stdout` is undesirable, you can disable this setting
    /// explicitly.
    pub fn emit_rerun_if_changed(mut self, enable: bool) -> Self {
        self.emit_rerun_if_changed = enable;
        self
    }

    /// Enable or disable directing service generation to providing a default implementation for service methods.
    /// When this is false all gRPC methods must be explicitly implemented.
    /// When this is true any unimplemented service methods will return 'unimplemented' gRPC error code.
    /// When this is true all streaming server request RPC types explicitly use tonic::codegen::BoxStream type.
    ///
    /// This defaults to `false`.
    pub fn generate_default_stubs(mut self, enable: bool) -> Self {
        self.generate_default_stubs = enable;
        self
    }

    /// Skips generating `impl Debug` for types
    pub fn skip_debug(mut self, path: impl AsRef<str>) -> Self {
        self.skip_debug.insert(path.as_ref().to_string());
        self
    }
}
