fn main() {
    kanamaru_build::ProstBuilder::default()
        .compile_protos(
            &["../../protos/posts.proto", "../../protos/posts/view.proto"],
            &["../../protos"],
        )
        .unwrap();
}
