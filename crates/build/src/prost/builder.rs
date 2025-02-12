use std::{
    collections::HashSet,
    ffi::OsString,
    io,
    path::{Path, PathBuf},
};

use prost_build::Config;

use crate::utils::Attributes;

use super::generator::ServiceGenerator;

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

    pub fn setup_config(self, config: &mut Config) {
        if let Some(out_dir) = self.out_dir.as_ref() {
            config.out_dir(out_dir);
        }
        if let Some(path) = self.file_descriptor_set_path.as_ref() {
            config.file_descriptor_set_path(path);
        }
        if self.skip_protoc_run {
            config.skip_protoc_run();
        }
        for (proto_path, rust_path) in self.extern_path.iter() {
            config.extern_path(proto_path, rust_path);
        }
        for (prost_path, attr) in self.field_attributes.iter() {
            config.field_attribute(prost_path, attr);
        }
        for (prost_path, attr) in self.type_attributes.iter() {
            config.type_attribute(prost_path, attr);
        }
        for (prost_path, attr) in self.message_attributes.iter() {
            config.message_attribute(prost_path, attr);
        }
        for (prost_path, attr) in self.enum_attributes.iter() {
            config.enum_attribute(prost_path, attr);
        }
        for prost_path in self.boxed.iter() {
            config.boxed(prost_path);
        }
        if let Some(ref paths) = self.btree_map {
            config.btree_map(paths);
        }
        if let Some(ref paths) = self.bytes {
            config.bytes(paths);
        }
        if self.compile_well_known_types {
            config.compile_well_known_types();
        }
        if let Some(path) = self.include_file.as_ref() {
            config.include_file(path);
        }
        if !self.skip_debug.is_empty() {
            config.skip_debug(&self.skip_debug);
        }

        for arg in self.protoc_args.iter() {
            config.protoc_arg(arg);
        }

        config.service_generator(self.service_generator());
    }

    /// Turn the builder into a `ServiceGenerator` ready to be passed to `prost-build`s
    /// `Config::service_generator`.
    pub fn service_generator(self) -> Box<dyn prost_build::ServiceGenerator> {
        Box::new(ServiceGenerator::new(self))
    }

    /// Compile the .proto files and execute code generation.
    pub fn compile_protos(
        self,
        protos: &[impl AsRef<Path>],
        includes: &[impl AsRef<Path>],
    ) -> io::Result<()> {
        self.compile_protos_with_config(Config::new(), protos, includes)
    }

    /// Compile the .proto files and execute code generation using a custom
    /// `prost_build::Config`. The provided config will be updated with this builder's config.
    pub fn compile_protos_with_config(
        self,
        mut config: Config,
        protos: &[impl AsRef<Path>],
        includes: &[impl AsRef<Path>],
    ) -> io::Result<()> {
        if self.emit_rerun_if_changed {
            for path in protos.iter() {
                println!("cargo:rerun-if-changed={}", path.as_ref().display())
            }

            for path in includes.iter() {
                // Cargo will watch the **entire** directory recursively. If we
                // could figure out which files are imported by our protos we
                // could specify only those files instead.
                println!("cargo:rerun-if-changed={}", path.as_ref().display())
            }
        }

        self.setup_config(&mut config);
        config.compile_protos(protos, includes)
    }

    /// Execute code generation from a file descriptor set.
    pub fn compile_fds(self, fds: prost_types::FileDescriptorSet) -> io::Result<()> {
        self.compile_fds_with_config(Config::new(), fds)
    }

    /// Execute code generation from a file descriptor set using a custom `prost_build::Config`.
    pub fn compile_fds_with_config(
        self,
        mut config: Config,
        fds: prost_types::FileDescriptorSet,
    ) -> io::Result<()> {
        self.setup_config(&mut config);
        config.compile_fds(fds)
    }
}
