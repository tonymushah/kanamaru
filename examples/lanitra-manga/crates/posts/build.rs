use lanitra_manga_proto_build::BuildConfig;

fn main() {
    BuildConfig {
        posts: false,
        posts_view: false,
        ..Default::default()
    }
    .builder()
    .compile_protos(
        &["../../protos/posts.proto", "../../protos/posts/view.proto"],
        &["../../protos"],
    )
    .unwrap();
}
