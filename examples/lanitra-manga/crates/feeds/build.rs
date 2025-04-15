use lanitra_manga_proto_build::BuildConfig;

fn main() {
    BuildConfig {
        feeds: false,
        ..Default::default()
    }
    .builder()
    .compile_protos(&["../../protos/feeds.proto"], &["../../protos"])
    .unwrap();
}
