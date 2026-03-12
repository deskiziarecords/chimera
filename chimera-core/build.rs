fn main() {
    #[cfg(feature = "build-info")]
    {
        use vergen::EmitBuilder;

        EmitBuilder::builder()
            .git_sha(true)
            .rustc_semver()
            .emit()
            .unwrap();
    }
}
