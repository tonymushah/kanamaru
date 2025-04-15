use lanitra_manga_proto_build::BuildConfig;

fn main() {
    BuildConfig {
        profiles: false,
        ..Default::default()
    }
    .builder()
    .compile_protos(&["../../protos/profiles.proto"], &["../../protos"])
    .unwrap();
}
