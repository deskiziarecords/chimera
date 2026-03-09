fn main() -> Result<(), Box<dyn std::error::Error>> {
    vergen::EmitBuilder::builder()
        .build_timestamp()
        .rustc_semver()
        .git_sha(true)
        .emit()?;
    Ok(())
}