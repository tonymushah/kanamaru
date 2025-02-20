fn main() {
    kanamaru_build::ProstBuilder::default()
        .compile_protos(&["../../protos/profiles.proto"], &["../../protos"])
        .unwrap();
}
