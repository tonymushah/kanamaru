use lanitra_manga_proto_build::BuildConfig;

fn main() {
    BuildConfig {
        auth: false,
        ..Default::default()
    }
    .builder()
    .compile_protos(&["../../protos/auth.proto"], &["../../protos"])
    .unwrap();
}
