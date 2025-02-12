fn main() -> anyhow::Result<()> {
    kanamaru_build::compile_protos("./protos/myprotos.proto")?;
    kanamaru_build::plugin_build();
    Ok(())
}
