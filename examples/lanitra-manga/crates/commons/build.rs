use lanitra_manga_proto_build::BuildConfig;

fn main() {
    BuildConfig {
        commons: false,
        ..Default::default()
    }
    .builder()
    .compile_protos(&["../../protos/commons.proto"], &["../../protos/"])
    .unwrap();
}
