fn main() -> anyhow::Result<()> {
    kanamaru_build::ProstBuilder::default().compile_protos(
        &["./protos/myprotos.proto", "./protos/commons.proto"],
        &["./protos/"],
    )?;
    kanamaru_build::plugin_build();
    Ok(())
}
