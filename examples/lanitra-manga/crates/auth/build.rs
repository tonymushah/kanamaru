fn main() {
    kanamaru_build::ProstBuilder::default()
        .compile_protos(&["../../protos/auth.proto"], &["../../protos"])
        .unwrap();
}
