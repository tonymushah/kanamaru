fn main() {
    kanamaru_build::ProstBuilder::default()
        .compile_protos(&["../../protos/commons.proto"], &["../../protos/"])
        .unwrap();
}
