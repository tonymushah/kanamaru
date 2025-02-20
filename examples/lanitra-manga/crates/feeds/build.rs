fn main() {
    kanamaru_build::ProstBuilder::default()
        .compile_protos(&["../../protos/feeds.proto"], &["../../protos"])
        .unwrap();
}
